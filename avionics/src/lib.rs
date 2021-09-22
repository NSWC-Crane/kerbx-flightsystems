use contracts::*;
use krpc_mars::protobuf::CodedOutputStream;
use krpc_mars::RPCClient;
use libkerbx::kerbx::Sheath_oneof_message::flightplan;
use libkerbx::kerbx::*;
use libkerbx::KerbxTransport;
use std::net::TcpStream;
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
    stage: u8,
    /// Last error message set
    error_message: String,
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
            stage: 0,
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
        let plan = self.flightplan.unwrap();

        // Flight plan must have at least one step
        if plan.step_count == 0 {
            return false;
        };

        // First plan step should be an engine ignite, i.e., launch
        if plan.steps[0].field_type != Step_ActionType::IGNITE {
            return false;
        };

        for step in self.flightplan.unwrap().steps {
            /*
            TODO: Flight plan steps should be checked against craft composition to make sure that
             each step can actually be executed by the constructed craft. For example, if an ignite
             is called and the crafts current "stage" isn't an engine, this validation should fail
             for safety reasons.
             */
        }

        true
    }

    pub fn set_error(&mut self, message: &str) {
        self.error_message = String::from(message);
    }

    // Flight plans should not have 255 stages or more
    #[invariant(self.stage < 255, "Stage count (u8) must not integer overflow")]
    pub fn inc_stage(&mut self) {
        self.stage += 1;
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
