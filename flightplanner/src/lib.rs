use libkerbx::kerbx::*;
use protobuf::{CodedInputStream, CodedOutputStream};
use std::io::Error;
use std::net::{TcpListener, TcpStream};

pub struct PlanningServer {
    ip: String,
    port: String,
    listener: TcpListener,
}

impl PlanningServer {
    /// Creates a new instance of the Planning Server on the provided ip and port
    pub fn new(ip: String, port: String) -> Result<PlanningServer, Error> {
        let listener = TcpListener::bind(format!("{}:{}", ip, port))?;

        Ok(PlanningServer { ip, port, listener })
    }

    pub fn process_input_messages(&self) {
        for stream in self.listener.incoming() {
            let mut stream = stream.unwrap();

            // Decode protobuf messages
            let mut input = CodedInputStream::new(&mut stream);
            let message: Sheath = match input.read_message() {
                Ok(x) => x,
                _ => {
                    eprintln!("Danger will robinson");
                    Sheath::new()
                }
            };
            match message.get_field_type() {
                Sheath_MessageType::COUNTCOWN => eprintln!("Countdown message!"),
                Sheath_MessageType::FLIGHTPLAN => eprintln!("Flightplan message!"),
                Sheath_MessageType::TELEMETRY => eprintln!("Telemetry message!"),
                Sheath_MessageType::WATCHDOG => eprintln!("Watchdog message!"),
                _ => eprintln!("Unknown message type."),
            }
        }
    }
}
