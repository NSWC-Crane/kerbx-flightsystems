use contracts::*;
use krpc_mars::protobuf::reflect::ProtobufValue;
use krpc_mars::protobuf::CodedOutputStream;
use krpc_mars::RPCClient;
use libkerbx::kerbx::Sheath_oneof_message::flightplan;
use libkerbx::kerbx::*;
use libkerbx::KerbxTransport;
use std::net::TcpStream;
use std::thread::current;
use std::time::SystemTime;

// Derive allows for boolean comparison of enums used in the contracts
#[derive(Eq, PartialEq)]
pub enum AvionicsState {
    OFF,
    POST,
    IDLE,
    READY,
    COUNTDOWN,
    INFLIGHT,
    LANDED,
    ERROR,
}

pub struct Avionics {
    state: AvionicsState,
    current_step: u32,     // Current step of the flight plan
    error_message: String, // Last error message set
    flight_planner: TcpStream,
    sensors: KerbxTransport,
    flightplan: Option<FlightPlan>,
}

impl Avionics {
    pub fn new(
        ip: String,
        port: String,
        sensors: KerbxTransport,
    ) -> Result<Avionics, std::io::Error> {
        let connection = TcpStream::connect(format!("{}:{}", ip, port))?;
        Ok(Avionics {
            state: AvionicsState::OFF,
            current_step: 0,
            error_message: String::from(""),
            flight_planner: connection,
            sensors,
            flightplan: None,
        })
    }

    /// Returns false if flightplan is invalide
    //TODO: Add in a proper result type for the flight plan validator
    #[requires(self.flightplan.is_some(), "Flightplan must exist to validate.")]
    pub fn validate_flightplan(&self) -> bool {
        // We shouldn't be calling this function if we have not already loaded the flight plan
        let plan: &FlightPlan = self.flightplan.as_ref().unwrap();

        // Flight plan must have at least one step
        if plan.step_count == 0 {
            eprintln!("Flight plan does not have at least one step.");
            return false;
        };

        // First plan step should be an engine ignite, i.e., launch
        if plan.steps[0].field_type != Step_ActionType::IGNITE {
            eprintln!("Flight plan does not begin with an IGNITE action.");
            return false;
        };

        // First trigger should be a Time trigger with 0 as the trigger time.
        if plan.steps[0].trigger.is_some() {
            if plan.steps[0].trigger.as_ref().unwrap().has_time() {
                if plan.steps[0].trigger.as_ref().unwrap().get_time().seconds != 0 {
                    eprintln!("First step must trigger on Time 0.");
                    return false;
                }
            } else {
                eprintln!("First step must use a Time trigger.");
                return false;
            }
        } else {
            eprintln!(
                "All steps must have triggers. Use a time of 0 if you have no valid condition.."
            );
            return false;
        }

        for step in self.flightplan.as_ref().unwrap().steps.as_ref() {
            /*
            TODO: Flight plan steps should be checked against craft composition to make sure that
             each step can actually be executed by the constructed craft. For example, if an ignite
             is called and the crafts current "stage" isn't an engine, this validation should fail
             for safety reasons.

             todo: Validate IGNITE action Type corresponds to a stage with an engine
             */
            if step.trigger.is_none() {
                eprintln!("Every step must have a trigger. Use a Time of 0 if you have no valid condition.");
                return false;
            }
        }

        true
    }

    pub fn flightplan_pop_step(mut self) -> Option<Step> {
        if let Some(mut plan) = self.flightplan {
            plan.steps.pop()
        } else {
            // todo: more robust/safe error handling
            panic!("Popping an empty flight plan.");
            None
        }
    }

    /// Executes a single step in the flight plan
    pub fn flightplan_exe_single_action(&self, step: &Step) {
        match &step.get_field_type() {
            Step_ActionType::REORIENT => {
                let pitch = step.get_position().get_pitch();
                let heading = step.get_position().get_yaw();

                self.sensors.set_auto_pilot(true);
                self.sensors.set_auto_pilot_direction(pitch, heading);
            }
            Step_ActionType::IGNITE => {
                // TODO: See validation of making sure the IGNITE type corresponds to an engine
                self.sensors.trigger_stage();
            }
            Step_ActionType::THROTTLELEVEL => {
                // Double call to get_throttle is a byproduct of a throttle action type also containing
                // a field of the same name. Mea culpa.
                self.sensors
                    .set_throttle(step.get_throttle().get_throttle());
            }
            Step_ActionType::COAST => {
                self.sensors.set_auto_pilot(false);
                //todo
            }
            Step_ActionType::NEXTSTAGE => {
                self.sensors.trigger_stage();
                //todo
            }
        }
    }

    /// Checks the trigger of a flight plan action returning true if the trigger is met.
    pub fn flightplan_check_trigger(&self, trigger: &Trigger) -> bool {
        if let Some(type_of_trigger) = &trigger.trigger_condition {
            match type_of_trigger {
                Trigger_oneof_trigger_condition::position(x) => {
                    //todo: Implement such that there is some room for error in the lat/lon checking in a way that makes sense
                    let current_lat = self.sensors.get_lat().expect("Error getting latitude.");
                    let min_lat = trigger.get_position().get_lat() - 10.0;
                    let max_lat = trigger.get_position().get_lat() + 10.0;

                    let current_lon = self.sensors.get_lon().expect("Error getting longitude.");
                    let min_lon = trigger.get_position().get_lon() - 10.0;
                    let max_lon = trigger.get_position().get_lon() + 10.0;

                    // If current_lat is within the latitude range, then we check the current lon.
                    // We'll return only true if both are within range.
                    return if min_lat <= current_lat && current_lat <= max_lat {
                        if min_lon <= current_lon && current_lon <= max_lon {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                }
                Trigger_oneof_trigger_condition::time(x) => {
                    let current_time = libkerbx::time().unwrap().seconds;

                    return if trigger.get_time().seconds < current_time {
                        true
                    } else {
                        false
                    };
                }
                Trigger_oneof_trigger_condition::alt(x) => {
                    //todo: Error handling
                    let current_alt = self.sensors.get_alt().expect("Error getting altitude.");

                    //todo: fix altitude checking such that the error is a function of the velocity/acceleration of the craft otherwise it's possible for the craft to be going so fast it misses its window
                    let minimum_alt = trigger.get_alt() - 10.0;
                    let maximum_alt = trigger.get_alt() + 10.0;

                    return if minimum_alt <= current_alt && current_alt <= maximum_alt {
                        true
                    } else {
                        false
                    };
                }
            }
        } else {
            //todo: In the real world we shouldn't panic here and instead fail gracefully as I don't
            // believe this state is reachable.
            panic!("Trigger without condition type! This should be impossible.")
        }

        false
    }

    pub fn set_error(&mut self, message: &str) {
        self.error_message = String::from(message);
    }

    // Flight plans should not have 255 stages or more
    pub fn inc_stage(&mut self) {
        self.current_step += 1;
    }

    #[requires(self.state == AvionicsState::OFF, "POST state only valid from OFF")]
    pub fn to_post(&mut self) {
        self.state = AvionicsState::POST;
    }

    #[requires(self.state == AvionicsState::POST, "IDLE state only valid from POST")]
    pub fn to_idle(&mut self) {
        self.state = AvionicsState::IDLE;
    }

    #[requires(self.state == AvionicsState::IDLE, "READY state only valid from IDLE")]
    #[requires(self.flightplan.is_some(), "Vessel must have a valid flight plan")]
    pub fn to_ready(&mut self) {
        self.state = AvionicsState::READY;
    }

    #[requires(self.state == AvionicsState::READY, "COUNTDOWN state only valid from READY")]
    pub fn to_countdown(&mut self) {
        self.state = AvionicsState::COUNTDOWN;
    }

    #[requires(self.state == AvionicsState::COUNTDOWN, "InFlight state only valid from COUNTDOWN")]
    pub fn to_inflight(&mut self) {
        self.state = AvionicsState::INFLIGHT;
    }

    #[requires(self.state == AvionicsState::INFLIGHT, "Landed state only valid from INFLIGHT")]
    pub fn to_landed(&mut self) {
        self.state = AvionicsState::LANDED;
    }

    // Error state is valid from all other states
    pub fn to_error(&mut self, message: &str) {
        self.state = AvionicsState::ERROR;
        self.error_message = String::from(message);
    }

    pub fn send_alive(&mut self) {
        let mut message = WatchDog::new();
        message.set_status(WatchDog_Status::ACKALIVE);
        message.set_time(libkerbx::time().unwrap());

        let mut wrapper = Sheath::new();
        wrapper.set_field_type(Sheath_MessageType::WATCHDOG);
        wrapper.set_watchdog(message);

        let mut output = CodedOutputStream::new(&mut self.flight_planner);
        if let Ok(()) = output.write_message_no_tag(&wrapper) {
            output.flush();
        } else {
            eprintln!("Error writing message to flight planner.");
        }
    }

    // Set throttle to 100 percent and disable flight stabalizers in prep for autopilot
    // TODO: Add error handling
    #[requires(self.state == AvionicsState::READY, "Cannot prep for launch unless flight plan is valid")]
    pub fn ready_for_launch(&mut self) {
        self.sensors.set_sas(false);
        self.sensors.set_rcs(false);
        self.sensors.set_throttle(1.0);
    }

    pub fn send_telemetry(&mut self) {
        // In a real world situation these errors should be passed up the stack and handled in
        // the main avionics loop. I'm running out of time as I write this, so this will just fail
        // early and hopefully not often.
        let mut message = Telemetry::new();
        message.set_lat(self.sensors.get_lat().expect("Error getting Latitude."));
        message.set_lon(self.sensors.get_lon().expect("Error getting Longitude."));
        message.set_alt(self.sensors.get_alt().expect("Error getting altitude."));
        message.set_yaw(self.sensors.get_heading().expect("Error getting heading."));
        message.set_pitch(self.sensors.get_pitch().expect("Error getting pitch."));
        message.set_roll(self.sensors.get_roll().expect("Error getting roll."));
        message.set_velocity(
            self.sensors
                .get_velocity()
                .expect("Error getting velocity."),
        );

        let mut time = Time::new();
        time.seconds = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Error getting system time.")
            .as_secs();
        message.set_time(time);

        let mut wrapper = Sheath::new();
        wrapper.set_field_type(Sheath_MessageType::TELEMETRY);
        wrapper.set_telemetry(message);

        let mut output = CodedOutputStream::new(&mut self.flight_planner);
        if let Ok(()) = output.write_message_no_tag(&wrapper) {
            output.flush();
        } else {
            eprintln!("Error writing message to flight planner.");
        }
    }
}
