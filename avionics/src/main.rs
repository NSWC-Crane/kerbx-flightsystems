use clap::{App, Arg};
use krpc_mars::RPCClient;
use libkerbx::kerbx::{Telemetry, Time};
use libkerbx::KerbxTransport;

use avionics::AvionicsState;
use std::time::{Duration, SystemTime};
use std::{error::Error, thread};

use avionics::Avionics;

static DEFAULT_FLIGHT_PLAN: &'static str = r#"{"step_count":1,"steps":[{"count":1,"field_type":"IGNITE","trigger":{"trigger_condition":{"time":{"seconds":0}}},"action":null}]}"#;

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments.
    let matches = App::new("KerbX Avionics Computer")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Avionics Computer for KerbX Rocketry Systems")
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
                .required(true)
                .help("IP Address of the Flight Planning Computer in Mission Control"),
        )
        .arg(
            Arg::with_name("plannerport")
                .short("o")
                .takes_value(true)
                .default_value("51961")
                .help("Port of the Flight Planning Computer in Mission Control"),
        )
        .get_matches();

    // Connect to KSP via krpc-rs -- this provides our sensor inputs and control surface outputs
    let server_address = format!(
        "{}:{}",
        matches.value_of("simip").unwrap(),
        matches.value_of("simport").unwrap()
    );
    let client = RPCClient::connect("Avionics Computer", server_address)
        .expect("Could not connect to KRPC Server.");

    // We obfuscate the RPC interface with our KerbxTransport wrapper.
    // TODO: Move RPC connections into the KerbxTransport Interface and just pass in the simip and simport
    let ship = KerbxTransport::new(client)?;

    // Area where we perform the Power-On-Self-Test Routine Operations //

    // Now Entering POST
    status.to_post();

    //***********************************************************************//

    // Area where we perform initial load of the flight plan from the flight planning computer //

    // Connect to flight planning server and send ALIVE
    let mut status = Avionics::new(
        String::from(matches.value_of("plannerip").unwrap()),
        String::from(matches.value_of("plannerport").unwrap()),
        ship,
    )?;
    status.send_alive();
    // Now wait for flight plan...
    status.to_idle();
    //**********************************************************************//

    // Area where we initiate launch //

    //*********************************************************************//

    // Flight control loop
    loop {
        status.send_alive();
        status.send_telemetry();
        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
