use nalgebra::{Vector3, Rotation3, Rotation};
use pancurses::{initscr, newwin, endwin, noecho, curs_set, doupdate, Input};
use krpc_mars::{RPCClient};
use std::{error::Error, thread, time};

use ksp_firecontrol::{space_center};

fn main() -> Result<(), Box<dyn Error>> {
    let main_window = initscr();
    noecho();
    curs_set(0);
    main_window.timeout(0);

    // Window constants used later for printing text    
    let lines = main_window.get_max_y();
    let cols = main_window.get_max_x();

    // Display our initial layout
    main_window.draw_box('|', '-');
    main_window.printw("Kerbel File Control");
    main_window.mvaddstr(lines - 1, 0, "Press 'q' to quit");
    main_window.timeout(0);
    main_window.noutrefresh();

    // Create the internal window we'll refresh with our output
    let output_window = newwin(lines - 6, cols - 4, 3, 2);
    output_window.timeout(0);
    output_window.noutrefresh();

    doupdate();

    // Connect to KSP via krpc-rs
    let client = krpc_mars::RPCClient::connect("Fire Control", "172.23.144.1:50000").expect("Could not connect to KRPC Server.");

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
            -nalgebra::angle_between(&horizon, &direction)
        } else {
            nalgebra::angle_between(&horizon, &direction)
        };
        
        // Calculate heading
        let heading = if horizon[2] < 0.0 {
            360.0 - nalgebra::angle_between(&horizon, &north)
        } else {
            nalgebra::angle_between(&horizon, &north)
        };

        let vessel_up = Vector3::new(vessel_up.0, vessel_up.1, vessel_up.2);
        let plane_normal = nalgebra::cross(&direction, &up);
        let roll = nalgebra::angle_between(&vessel_up, &plane_normal);
        let roll = if up[0] > 0.0 {
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
        
        output_window.mvaddstr(1, 1, format!("Yaw: {:?}", heading));
        output_window.mvaddstr(2, 1, format!("Pitch: {}", pitch));
        output_window.mvaddstr(3, 1, format!("Roll: {}", roll));
        output_window.mvaddstr(4, 1, format!("Lat: {}", lat));
        output_window.mvaddstr(5, 1, format!("Lon: {}", lon));
        output_window.mvaddstr(6, 1, format!("Alt: {}", alt));
        output_window.mvaddstr(7, 1, format!("Direction: {:?}", direction));

        if let Some(Input::Character('q')) = main_window.getch() {
            break;
        }

        // Window refreshes from the bottom up
        main_window.noutrefresh();
        output_window.noutrefresh();
        doupdate();

        thread::sleep(time::Duration::from_millis(100));
    }

    // Cleanup after ourselves
    endwin();

    Ok(())
}