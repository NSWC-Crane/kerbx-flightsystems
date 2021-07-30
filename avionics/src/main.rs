use clap::{App, Arg};
use krpc_mars::RPCClient;
use libkerbx::kerbx::{Telemetry, Time};
use libkerbx::KerbxTransport;

use std::time::{Duration, SystemTime};
use std::{error::Error, thread};

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
                .default_value("1797")
                .help("Port of the Flight Planning Computer in Mission Control"),
        )
        .get_matches();

    // Connect to KSP via krpc-rs
    let server_address = format!(
        "{}:{}",
        matches.value_of("simip").unwrap(),
        matches.value_of("simport").unwrap()
    );
    let client = RPCClient::connect("Avionics Computer", server_address)
        .expect("Could not connect to KRPC Server.");

    let ship = KerbxTransport::new(client)?;

    loop {
        let telemetry = build_telemetry_packet(&ship)?;
        println!("[[[[ {} ]]]]", telemetry.get_time().get_seconds());
        println!("Roll:{}", telemetry.get_roll());
        println!("Pitch:{}", telemetry.get_pitch());
        println!("Heading:{}", telemetry.get_yaw());
        println!("Lat:{}", telemetry.get_lat());
        println!("Lon:{}", telemetry.get_lon());
        println!("Alt:{}", telemetry.get_alt());
        println!("Velocity:{}", telemetry.get_velocity());
        println!("[[[[ END ]]]]");
        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}

fn build_telemetry_packet(sensors: &KerbxTransport) -> Result<Telemetry, Box<dyn Error>> {
    let mut packet = Telemetry::new();
    packet.lat = sensors.get_lat()?;
    packet.lon = sensors.get_lon()?;
    packet.alt = sensors.get_alt()?;
    packet.yaw = sensors.get_heading()?;
    packet.pitch = sensors.get_pitch()?;
    packet.roll = sensors.get_roll()?;
    packet.velocity = sensors.get_velocity()?;

    let mut time = Time::new();
    time.seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    packet.set_time(time);

    Ok(packet)
}
