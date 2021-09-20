use clap::{App, Arg};
use libkerbx::kerbx::*;
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

    // Output file is a required argument
    let output = File::open(matches.value_of("output-file").unwrap())
        .expect("Could not open flightplan output file.");

    // Created our overall flight plan object
    let plan = FlightPlan::new();

    // Now add our individual steps
    let mut steps: Vec<Step> = Vec::new();

    steps.push(gen_other_step(
        1,
        Step_ActionType::IGNITE,
        gen_time_trigger(0),
    ));
}

/// Generates a step that tells the craft to change the throttle level to a certain value between
/// 0.0 and 1.0. Any values less than 0 will be locked to 0.0. Any values greater than 1.0 will be
/// locked to 1.0
fn gen_throttle_step(count: u32, throttle_level: f32, trigger: Trigger) -> Step {
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
fn gen_reorient_step(count: u32, roll: f32, pitch: f32, yaw: f32, trigger: Trigger) -> Step {
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
fn gen_other_step(count: u32, actiontype: Step_ActionType, trigger: Trigger) -> Step {
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
fn gen_time_trigger(seconds_after_epoch: u64) -> Trigger {
    let mut trigger = Trigger::new();

    let mut trigger_time = Time::new();
    trigger_time.seconds = seconds_after_epoch;

    trigger.set_time(trigger_time);

    trigger
}

/// Generates a trigger that will occur when the craft reaches a certain altitude on kerbin.
/// altitude: distance in km to trigger.
fn gen_alt_trigger(altitude: f64) -> Trigger {
    let mut trigger = Trigger::new();
    trigger.set_alt(altitude);
    trigger
}

/// Generates a trigger that will occur when the craft reaches a certain latitude and longitude on
/// Kerbin.
fn gen_pos_trigger(lat: f64, lon: f64) -> Trigger {
    let mut trigger = Trigger::new();

    let mut position = Position::new();
    position.set_lat(lat);
    position.set_lon(lon);

    trigger.set_position(position);

    trigger
}
