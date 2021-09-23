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

use crate::kerbx::*;
use protobuf::RepeatedField;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::fs::File;
use std::io::{BufReader, BufWriter};

//TODO: Handling with proper custom Error type
pub fn load_from_string(flightplan: &str) -> FlightPlan {
    serde_json::from_str(flightplan).expect("Error loading flight plan.")
}

//TODO: Handling with proper custom Error type
/// Loads a flight plan from a specified file
/// filename: Full canonical path to the flightplan to load
pub fn load_from_file(filename: &str) -> FlightPlan {
    let mut file = File::open(filename).expect("Could not open flight plan file.");

    serde_json::from_reader(BufReader::new(file)).expect("Error loading flight plan from file.")
}

/// Writes a flight plan from memory to specified file.
/// filename: File to write to.
/// &plan: Reference to flight plan object in memory
pub fn write_to_file(filename: &str, plan: &FlightPlan) {
    let mut file = File::create(filename).expect("Could not open flight plan file.");

    serde_json::to_writer(BufWriter::new(file), &plan);
}

/// Generates a step that tells the craft to change the throttle level to a certain value between
/// 0.0 and 1.0. Any values less than 0 will be locked to 0.0. Any values greater than 1.0 will be
/// locked to 1.0
pub fn gen_throttle_step(count: u32, throttle_level: f32, trigger: Trigger) -> Step {
    let mut step = Step::new();
    step.set_field_type(Step_ActionType::THROTTLELEVEL);

    let mut throttle = ThrottleLevel::new();
    throttle.set_throttle(if throttle_level > 1.0 {
        1.0
    } else if throttle_level < 0.0 {
        0.0
    } else {
        throttle_level
    });

    step.set_count(count);
    step.set_throttle(throttle);
    step.set_trigger(trigger);

    step
}

/// Generates a step that tells the craft to reorient to a certain roll, pitch, and yaw.
pub fn gen_reorient_step(count: u32, roll: f32, pitch: f32, yaw: f32, trigger: Trigger) -> Step {
    let mut step = Step::new();
    step.set_field_type(Step_ActionType::REORIENT);

    let mut reorient = Reorient::new();
    reorient.set_pitch(pitch);
    reorient.set_roll(roll);
    reorient.set_yaw(yaw);

    step.set_count(count);
    step.set_position(reorient);
    step.set_trigger(trigger);

    step
}

/// Generates one of the other step types (COAST, IGNITE, NEXTSTAGE). These do not have associated
/// arguments and thus can all be set with the same function
pub fn gen_other_step(count: u32, actiontype: Step_ActionType, trigger: Trigger) -> Step {
    let mut step = Step::new();
    if let Step_ActionType::REORIENT | Step_ActionType::IGNITE | Step_ActionType::COAST = actiontype
    {
        step.set_count(count);
        step.set_field_type(actiontype);
        step.set_trigger(trigger);
    } else {
        panic!("Flightplan Compilation Error. THROTTLELEVEL or REORIENT used with gen_other_step.")
    }
    step
}

/// Generates a trigger that will occur when a certain time is reached. Time must be provided in
/// seconds after the UNIX epoch. If passed a value less than the current time in seconds after
/// the epoch, the trigger will occur immediately.
pub fn gen_time_trigger(seconds_after_epoch: u64) -> Trigger {
    let mut trigger = Trigger::new();

    let mut trigger_time = Time::new();
    trigger_time.seconds = seconds_after_epoch;

    trigger.set_time(trigger_time);

    trigger
}

/// Generates a trigger that will occur when the craft reaches a certain altitude on kerbin.
/// altitude: distance in km to trigger.
pub fn gen_alt_trigger(altitude: f64) -> Trigger {
    let mut trigger = Trigger::new();
    trigger.set_alt(altitude);
    trigger
}

/// Generates a trigger that will occur when the craft reaches a certain latitude and longitude on
/// Kerbin.
pub fn gen_pos_trigger(lat: f64, lon: f64) -> Trigger {
    let mut trigger = Trigger::new();

    let mut position = Position::new();
    position.set_lat(lat);
    position.set_lon(lon);

    trigger.set_position(position);

    trigger
}

pub fn gen_flightplan_from_steps(steps: Vec<Step>) -> FlightPlan {
    let steps = RepeatedField::from_vec(steps);

    let mut plan = FlightPlan::new();
    plan.set_step_count(steps.len() as u32);
    plan.set_steps(steps);

    plan
}
