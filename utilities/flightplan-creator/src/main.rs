use clap::{App, Arg};
use libkerbx::{flightplan::*, kerbx::*};
use serde::Serialize;
use serde_json::{Result, Value};
use std::fs::File;
use std::io::BufWriter;

fn main() {
    // Parse command line arguments.
    let matches = App::new("KerbX Flightplan Created")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Creates and saves a KerbX supported Flight Plan")
        .arg(
            Arg::with_name("output-file")
                .short("o")
                .takes_value(true)
                .required(true)
                .help("Filename to save compiled flight plan."),
        )
        .get_matches();

    // Created our overall flight plan object
    let mut plan = FlightPlan::new();

    // Now add our individual steps
    let mut steps: Vec<Step> = Vec::new();

    steps.push(gen_other_step(
        1,
        Step_ActionType::IGNITE,
        gen_time_trigger(0),
    ));

    let plan = gen_flightplan_from_steps(steps);

    write_to_file(matches.value_of("output-file").unwrap(), &plan);
}
