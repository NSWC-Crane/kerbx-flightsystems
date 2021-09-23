use clap::{App, Arg};
use krpc_mars::RPCClient;
use libkerbx::kerbx::{Telemetry, Time};
use libkerbx::KerbxTransport;

use avionics::AvionicsState;
use std::time::{Duration, SystemTime};
use std::{error::Error, thread};

use avionics::Avionics;

const FIVEHUNDREDHZ_IN_MS: u64 = 2;

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
    // "Boot UP" our flight computer
    let mut status = Avionics::new(
        String::from(matches.value_of("plannerip").unwrap()),
        String::from(matches.value_of("plannerport").unwrap()),
        ship,
    )?;
    // Now Entering POST
    status.to_post();

    // TODO: Check for POST failure and set ERROR message.

    //***********************************************************************//
    // Area where we perform initial load of the flight plan from the flight planning computer //
    status.send_alive();
    // Now wait for flight plan...
    status.to_idle();

    // Here's where we block for receipt of the flight plan
    // TODO: Receive flight plan over the network and validate flight plan
    status.load_flightplan();

    status.to_ready();

    // Prepare craft for launch
    status.ready_for_launch();
    //**********************************************************************//

    // Area where we initiate launch //
    status.to_countdown();

    for tick in (1..11).rev() {
        println!("Launching in {} seconds...", tick);

        // TODO: This should be a spinlock waiting for abort commands from the flight planning system
        // TODO: Check craft for errors...
        thread::sleep(Duration::from_secs(1));
    }

    // Execute the first step!
    let first = status.flightplan_pop_step().expect("First step empty.");
    status.flightplan_exe_single_action(&first);
    //*********************************************************************//
    // todo: Improve robustness of flight control loop
    while status.get_state() != &AvionicsState::LANDED {
        while let Some(step) = status.flightplan_pop_step() {
            // Keep looking until our Trigger is met and we execute our step
            let trigger = step.trigger.get_ref();
            loop {
                if status.flightplan_check_trigger(trigger) {
                    status.flightplan_exe_single_action(&step);
                }
                //
                status.send_alive();
                status.send_telemetry();
                // Effectively this is sending the environment at 500hz
                thread::sleep(Duration::from_millis(FIVEHUNDREDHZ_IN_MS));
            }
        }

        // We're out of flight planning steps but keep sending our watchdogs and telemetry
        status.send_alive();
        status.send_telemetry();

        // Todo: Implement check for transition to to_landed state -- could just cheat and use krpc

        thread::sleep(Duration::from_millis(FIVEHUNDREDHZ_IN_MS));
    }

    Ok(())
}
