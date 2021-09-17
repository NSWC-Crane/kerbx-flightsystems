use contracts::*;
use krpc_mars::protobuf::CodedOutputStream;
use libkerbx::kerbx::*;
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
}

impl Avionics {
    pub fn new(ip: String, port: String) -> Result<Avionics, std::io::Error> {
        let connection = TcpStream::connect(format!("{}:{}", ip, port))?;
        Ok(Avionics {
            state: AvionicsState::Off,
            stage: 0,
            error_message: String::from(""),
            flight_planner: connection,
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
        output.write_message_no_tag(&wrapper).unwrap();
        output.flush();
    }
}
