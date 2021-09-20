use crate::kerbx::FlightPlan;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::fs::File;
use std::io::BufReader;

//TODO: Handling with proper custom Error type
pub fn load_from_string(flightplan: String) -> FlightPlan {
    serde_json::from_str(flightplan.as_str()).expect("Error loading flight plan.")
}

//TODO: Handling with proper custom Error type
/// Loads a flight plan from a specified file
/// filename: Full canonical path to the flightplan to load
pub fn load_from_file(filename: String) -> FlightPlan {
    let mut file = File::open(filename).expect("Could not open flight plan file.");

    serde_json::from_reader(BufReader::new(file)).expect("Error loading flight plan from file.")
}
