use nalgebra::{Vector3};
use krpc_mars::{RPCClient};
use clap::{Arg,App};
use termion::{clear, cursor, terminal_size, raw::IntoRawMode, raw::RawTerminal};

use std::{error::Error, thread, time};
use std::io::{Read, Write, stdout};

use ksp_firecontrol::{space_center};

const HORZ_BOUNDARY: &'static str = "─";
const VERT_BOUNDARY: &'static str = "│";
const WIN_TITLE: &'static str = "Kerbal Fire Control";
const QUIT_MSG: &'static str = "Press 'q' to quit.";

fn draw_window<W: Write>(term: &mut RawTerminal<W>) -> Result<(),std::io::Error> {
    // Clear the window
    write!(term, "{}", clear::All).unwrap();

    // Get window size
    let (cols, rows) = terminal_size()?;

    // Print top and bottom boarders
    for x in 1..=cols {
        // Print top
        write!(term, "{}{}", cursor::Goto(x,1), HORZ_BOUNDARY);

        // Print bottom
        write!(term, "{}{}", cursor::Goto(x,rows), HORZ_BOUNDARY);
    }

    // Print side boarders 
    for y in 2..rows {
        // Print left
        write!(term, "{}{}", cursor::Goto(1,y), VERT_BOUNDARY);

        // Print right
        write!(term, "{}{}", cursor::Goto(cols,y), VERT_BOUNDARY);
    }

    // Print our title
    write!(term, "{}{}", cursor::Goto(5,1), WIN_TITLE);

    // Print our bottom quit message
    write!(term, "{}{}", cursor::Goto(5,rows), QUIT_MSG);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {

    // Parse command line arguments.
    let matches = App::new("KSP Fire Control")
    .version(env!("CARGO_PKG_VERSION"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .about("Kerbal Space Program Mission Computer")
    .arg(
        Arg::with_name("ip")
        .short("i")
        .takes_value(true)
        .required(true)
        .help("IP Address of the KRPC Server")
    ).arg(
        Arg::with_name("port")
        .short("p")
        .takes_value(true)
        .default_value("50000")
        .help("Port of the KRPC Server")
    ).get_matches();


    // Enter Raw Terminal mode for the app
    let mut stdout = stdout().into_raw_mode().unwrap();

    // Draw our border
    draw_window(&mut stdout)?;

    // Connect to KSP via krpc-rs
    let server_address = format!("{}:{}", matches.value_of("ip").unwrap(), matches.value_of("port").unwrap());
    let client = RPCClient::connect("Fire Control", server_address).expect("Could not connect to KRPC Server.");

    // Main loop for handling information from ksp
    loop {
        
        let vessel = client.mk_call(&space_center::get_active_vessel())?;
        
        let surf_ref_frame = client.mk_call(&vessel.get_surface_reference_frame())?;
        let vessel_ref_frame = client.mk_call(&vessel.get_reference_frame())?;

        // Get values for calculating lat/lon
        let orb_ref_frame = client.mk_call(&vessel.get_orbital_reference_frame())?;
        let orbit = client.mk_call(&vessel.get_orbit())?;
        let planet = client.mk_call(&orbit.get_body())?;

        // Get current vessel direction
        let direction = client.mk_call(&vessel.direction(&surf_ref_frame))?;
        let direction = Vector3::new(direction.0, direction.1, direction.2);

        // Constants leveraged for the final calculation.
        let horizon = Vector3::new(0.0, direction[1], direction[2]);
        let north = Vector3::new(0.0, 1.0, 0.0);
        let up = Vector3::new(1.0, 0.0, 0.0);

        // This line is causing Protobuf(WireError(UnecpectedEof)) errors.
        let vessel_up = client.mk_call(&space_center::transform_direction((0.0, 0.0, -1.0), &vessel_ref_frame, &surf_ref_frame))?;

        // Calculate pitch
        let pitch = if direction[0] < 0.0 {
            -to_degrees(horizon.angle(&direction))
        } else {
            to_degrees(horizon.angle(&direction))
        };
        
        // Calculate heading
        let heading = if horizon[2] < 0.0 {
            360.0 - to_degrees(horizon.angle(&north))
        } else {
            to_degrees(horizon.angle(&north))
        };

        let vessel_up = Vector3::new(vessel_up.0, vessel_up.1, vessel_up.2);
        let plane_normal = direction.cross(&up);
        let roll = to_degrees(vessel_up.angle(&plane_normal));
        let roll = if vessel_up[0] > 0.0 {
            roll * -1.0
        } else {
            if roll < 0.0 {
                roll + 180.0
            } else {
                roll - 180.0
            }
        };


        // Get current vessel position
        let position = client.mk_call(&vessel.position(&orb_ref_frame))?;
        let lat = client.mk_call(&planet.latitude_at_position(position, &orb_ref_frame))?;
        let lon = client.mk_call(&planet.longitude_at_position(position, &orb_ref_frame))?;
        let alt = client.mk_call(&planet.altitude_at_position(position, &orb_ref_frame))?;
        /*
        output_window.mvaddstr(1, 1, format!("Yaw: {}", heading));
        output_window.mvaddstr(2, 1, format!("Pitch: {}", pitch));
        output_window.mvaddstr(3, 1, format!("Roll: {}", roll));
        output_window.mvaddstr(4, 1, format!("Lat: {}", lat));
        output_window.mvaddstr(5, 1, format!("Lon: {}", lon));
        output_window.mvaddstr(6, 1, format!("Alt: {}", alt));

        if let Some(Input::Character('q')) = main_window.getch() {
            break;
        }

        // Window refreshes from the bottom up
        main_window.noutrefresh();
        output_window.noutrefresh();
        doupdate();
*/
        thread::sleep(time::Duration::from_millis(100));
    }

    // Cleanup after ourselves
    // endwin();

    Ok(())
}

fn to_degrees(val: f64) -> f64 {
    val * (180.0 / std::f64::consts::PI)
}