/*
* =================================================================================================
*
*                                      PUBLIC DOMAIN NOTICE
*                           Naval Surface Warfare Center - Crane Division
*
*  This software is a "United States Government Work" under the terms of the United States
*  Copyright Act. It was written as part of the author's official duties as a United States
*  Government employee and thus cannot be copyrighted. This software/database is freely available
*  to the public for use. Naval Surface Warfare Center - Crane Division (NSWC-CD) and the U.S.
*  Government have not places any restriction on its use or reproduction.
*
*  Although all reasonable efforts have been taken to ensure the accuracy and reliability of the
*  software and data, NSWC-CD and the U.S. Government do not and cannot warrant the performance or
*  results that may be obtained by using this software or data. NSWC-CD and the U.S. Government
*  disclaim all warranties, express or implied, including warranties of performance,
*  merchantability or fitness for any particular purpose.
*
*  Please cite the author in any work or product based on this material.
*
* =================================================================================================
*/

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
