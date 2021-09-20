use crate::kerbx::FlightPlan;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::fs::File;
use std::io::{BufReader, BufWriter};

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

/// Writes a flight plan from memory to specified file.
/// filename: File to write to.
/// &plan: Reference to flight plan object in memory
pub fn write_to_file(filename: String, plan: &FlightPlan) {
    let mut file = File::open(filename).expect("Could not open flight plan file.");

    serde_json::to_writer(BufWriter::new(file), &plan);
}
