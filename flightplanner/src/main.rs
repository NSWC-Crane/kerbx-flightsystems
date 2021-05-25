use clap::{App, Arg};
use krpc_mars::RPCClient;
use termion::{async_stdin, clear, cursor, raw::IntoRawMode, raw::RawTerminal, terminal_size};

use std::io::{stdout, Read, Write};
use std::{error::Error, thread, time};

use libkerbx::KerbxTransport;

const HORZ_BOUNDARY: &'static str = "─";
const VERT_BOUNDARY: &'static str = "│";
const WIN_TITLE: &'static str = "KerbX Flight Planner";
const QUIT_MSG: &'static str = "Press 'q' to quit.";

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments.
    let matches = App::new("KerbX Flight Planner")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("KerbX Flight Planning Mission Computer")
        .arg(
            Arg::with_name("simip")
                .short("i")
                .takes_value(true)
                .required(true)
                .help("IP Address of the KRPC (Sim) Server"),
        )
        .arg(
            Arg::with_name("simport")
                .short("p")
                .takes_value(true)
                .default_value("50000")
                .help("Port of the KRPC (SIM) Server"),
        )
        .arg(
            Arg::with_name("avionicsip")
                .short("a")
                .takes_value(true)
                .required(true)
                .help("IP Address of the Avionics Computer on the Transport"),
        )
        .arg(
            Arg::with_name("avionicsport")
                .short("o")
                .takes_value(true)
                .default_value("1797")
                .help("Port of the Avionics Computer on the Transport"),
        )
        .get_matches();

    // Enter Raw Terminal mode for the app
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut stdin = async_stdin().bytes();

    // Draw our border
    draw_window(&mut stdout)?;

    // Connect to KSP via krpc-rs
    let server_address = format!(
        "{}:{}",
        matches.value_of("simip").unwrap(),
        matches.value_of("simport").unwrap()
    );
    let client = RPCClient::connect("Flight Planner", server_address)
        .expect("Could not connect to KRPC Server.");

    let avionicsip = matches.value_of("avionicsip").unwrap();
    let avionicsport = matches.value_of("avionicsport").unwrap();

    // Eventually this will need to pull telemetry straight from the avionics computer on the craft
    // but that requires the comms protocol to be in place
    let transport_craft = KerbxTransport::new(client)?;

    // Main loop for handling information from ksp
    loop {
        let pitch = transport_craft.get_pitch()?;
        let heading = transport_craft.get_heading()?;
        let roll = transport_craft.get_roll()?;
        let lat = transport_craft.get_lat()?;
        let lon = transport_craft.get_lon()?;
        let alt = transport_craft.get_alt()?;

        // Quit on recieving q
        let b = stdin.next();
        if let Some(Ok(b'q')) = b {
            break;
        }

        mvaddstr(&mut stdout, 3, 3, format!("Yaw: {}", heading).as_str())?;
        mvaddstr(&mut stdout, 3, 4, format!("Pitch: {}", pitch).as_str())?;
        mvaddstr(&mut stdout, 3, 5, format!("Roll: {}", roll).as_str())?;
        mvaddstr(&mut stdout, 3, 6, format!("Lat: {}", lat).as_str())?;
        mvaddstr(&mut stdout, 3, 7, format!("Lon: {}", lon).as_str())?;
        mvaddstr(&mut stdout, 3, 8, format!("Alt: {}", alt).as_str())?;

        thread::sleep(time::Duration::from_millis(100));
    }

    // Cleanup after ourselves
    write!(stdout, "{}", clear::All).unwrap();

    Ok(())
}

fn draw_window<W: Write>(term: &mut RawTerminal<W>) -> Result<(), std::io::Error> {
    // Clear the window
    write!(term, "{}", clear::All).unwrap();

    // Get window size
    let (cols, rows) = terminal_size()?;

    // Print top and bottom boarders
    for x in 1..=cols {
        // Print top
        write!(term, "{}{}", cursor::Goto(x, 1), HORZ_BOUNDARY)?;

        // Print bottom
        write!(term, "{}{}", cursor::Goto(x, rows), HORZ_BOUNDARY)?;
    }

    // Print side boarders
    for y in 2..rows {
        // Print left
        write!(term, "{}{}", cursor::Goto(1, y), VERT_BOUNDARY)?;

        // Print right
        write!(term, "{}{}", cursor::Goto(cols, y), VERT_BOUNDARY)?;
    }

    // Print our title
    mvaddstr(term, 5, 1, WIN_TITLE)?;

    // Print our bottom quit message
    mvaddstr(term, 5, rows, QUIT_MSG)?;

    term.flush()?;

    Ok(())
}

fn mvaddstr<W: Write>(
    term: &mut RawTerminal<W>,
    col: u16,
    row: u16,
    text: &str,
) -> Result<(), std::io::Error> {
    write!(term, "{}{}{}", cursor::Goto(col, row), text, cursor::Hide)?;
    Ok(())
}

/*








fn main() -> Result<(), Box<dyn Error>> {

    /
}

fn to_degrees(val: f64) -> f64 {
    val * (180.0 / std::f64::consts::PI)
}
 */
