use contracts::*;
use krpc_mars::protobuf::CodedOutputStream;
use krpc_mars::RPCClient;
use libkerbx::kerbx::*;
use libkerbx::KerbxTransport;
use std::net::TcpStream;
use std::time::SystemTime;

// Derive allows for boolean comparison of enums used in the contracts
#[derive(Eq, PartialEq)]
pub enum AvionicsState {
    Off,
    POST,
    Idle,
    Ready,
    Countdown,
    InFlight,
    Landed,
    Error,
}

pub struct Avionics {
    state: AvionicsState,
    stage: u8,
    /// Last error message set
    error_message: String,
    flight_planner: TcpStream,
    sensors: KerbxTransport,
}

impl Avionics {
    pub fn new(
        ip: String,
        port: String,
        sensors: KerbxTransport,
    ) -> Result<Avionics, std::io::Error> {
        let connection = TcpStream::connect(format!("{}:{}", ip, port))?;
        Ok(Avionics {
            state: AvionicsState::Off,
            stage: 0,
            error_message: String::from(""),
            flight_planner: connection,
            sensors,
        })
    }

    pub fn set_error(&mut self, message: &str) {
        self.error_message = String::from(message);
    }

    // Flight plans should not have 255 stages or more
    #[invariant(self.stage < 255, "Stage count (u8) must not integer overflow")]
    pub fn inc_stage(&mut self) {
        self.stage += 1;
    }

    #[requires(self.state == AvionicsState::Off, "POST state only valid from Off")]
    pub fn to_post(&mut self) {
        self.state = AvionicsState::POST;
    }

    #[requires(self.state == AvionicsState::POST, "Idle state only valid from POST")]
    pub fn to_idle(&mut self) {
        self.state = AvionicsState::Idle;
    }

    #[requires(self.state == AvionicsState::Idle, "Ready state only valid from Idle")]
    pub fn to_ready(&mut self) {
        self.state = AvionicsState::Ready;
    }

    #[requires(self.state == AvionicsState::Ready, "Countdown state only valid from Ready")]
    pub fn to_countdown(&mut self) {
        self.state = AvionicsState::Countdown;
    }

    #[requires(self.state == AvionicsState::Countdown, "InFlight state only valid from Countdown")]
    pub fn to_inflight(&mut self) {
        self.state = AvionicsState::InFlight;
    }

    #[requires(self.state == AvionicsState::InFlight, "Landed state only valid from InFlight")]
    pub fn to_landed(&mut self) {
        self.state = AvionicsState::Landed;
    }

    // Error state is valid from all other states
    pub fn to_error(&mut self, message: &str) {
        self.state = AvionicsState::Error;
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
