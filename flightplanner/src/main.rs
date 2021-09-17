use clap::{App, Arg};
use krpc_mars::RPCClient;
use termion::{async_stdin, clear, cursor, raw::IntoRawMode, raw::RawTerminal, terminal_size};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::TryRecvError;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::task;

use std::io::{stdout, Read, Write};
use std::sync::Arc;
use std::{error::Error, thread, time};

use flightplanner::PlanningServer;
use libkerbx::kerbx::{Sheath, Sheath_MessageType};
use libkerbx::space_center::orbit_static_reference_plane_normal;
use libkerbx::KerbxTransport;

const HORZ_BOUNDARY: &'static str = "─";
const VERT_BOUNDARY: &'static str = "│";
const WIN_TITLE: &'static str = "KerbX Flight Planner";
const QUIT_MSG: &'static str = "Press 'q' to quit.";

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
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
            Arg::with_name("plannerip")
                .short("a")
                .takes_value(true)
                .default_value("127.0.0.1")
                .help("IP Address to start the Flight Planner TCP Server on"),
        )
        .arg(
            Arg::with_name("plannerport")
                .short("o")
                .takes_value(true)
                .default_value("51961")
                .help("Port to start the Flight Planner TCP Server on"),
        )
        .get_matches();

    // Enter Raw Terminal mode for the app
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut stdin = async_stdin().bytes();

    // Draw our border
    draw_window(&mut stdout)?;

    /*
        // Connect to KSP via krpc-rs
        let krpc_server_address = format!(
            "{}:{}",
            matches.value_of("simip").unwrap(),
            matches.value_of("simport").unwrap()
        );
        let client = RPCClient::connect("Flight Planner", krpc_server_address)
            .expect("Could not connect to KRPC Server.");

        // Eventually this will need to pull telemetry straight from the avionics computer on the craft
        // but that requires the comms protocol to be in place
        let transport_craft = KerbxTransport::new(client)?;
    */
    // Start up the flight planning server
    let plan_server = PlanningServer::new(
        String::from(matches.value_of("plannerip").unwrap()),
        String::from(matches.value_of("plannerport").unwrap()),
    )
    .await?;

    // Create the communications channels between async processes
    let (tx, mut rx_gui) = broadcast::channel(32);

    task::spawn(PlanningServer::recv_and_decode(plan_server, tx));

    // Main loop for handling information from ksp
    loop {
        // Quit on recieving q
        let b = stdin.next();
        if let Some(Ok(b'q')) = b {
            break;
        }

        if let Ok(message) = rx_gui.try_recv() {
            match message.field_type {
                Sheath_MessageType::WATCHDOG => {
                    let watchdog = message.get_watchdog();
                    mvaddstr(
                        &mut stdout,
                        3,
                        10,
                        format!("Watchdog sent at: {}.", watchdog.get_time().get_seconds())
                            .as_str(),
                    )?;
                }
                Sheath_MessageType::TELEMETRY => {
                    let telemetry = message.get_telemetry();
                    mvaddstr(
                        &mut stdout,
                        3,
                        3,
                        format!("Yaw: {}", telemetry.get_yaw()).as_str(),
                    )?;
                    mvaddstr(
                        &mut stdout,
                        3,
                        4,
                        format!("Pitch: {}", telemetry.get_pitch()).as_str(),
                    )?;
                    mvaddstr(
                        &mut stdout,
                        3,
                        5,
                        format!("Roll: {}", telemetry.get_roll()).as_str(),
                    )?;
                    mvaddstr(
                        &mut stdout,
                        3,
                        6,
                        format!("Lat: {}", telemetry.get_lat()).as_str(),
                    )?;
                    mvaddstr(
                        &mut stdout,
                        3,
                        7,
                        format!("Lon: {}", telemetry.get_lon()).as_str(),
                    )?;
                    mvaddstr(
                        &mut stdout,
                        3,
                        8,
                        format!("Alt: {}", telemetry.get_alt()).as_str(),
                    )?;
                }
                Sheath_MessageType::EMPTY => {
                    mvaddstr(
                        &mut stdout,
                        3,
                        11,
                        format!(
                            "Empty Sheath Sent at {}",
                            libkerbx::time().unwrap().get_seconds()
                        )
                        .as_str(),
                    )?;
                }
                _ => {
                    mvaddstr(
                        &mut stdout,
                        3,
                        12,
                        format!(
                            "Unsupported Sheath Type Sent at {}",
                            libkerbx::time().unwrap().get_seconds()
                        )
                        .as_str(),
                    )?;
                }
            }
        } else {
            // This can just mean there's nothing to receive
            /*
            mvaddstr(
                &mut stdout,
                3,
                20,
                format!(
                    "try_recv error at {}",
                    libkerbx::time().unwrap().get_seconds()
                )
                .as_str(),
            )?;
             */
        }
        stdout.flush();
        //thread::sleep(time::Duration::from_millis(100));
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
