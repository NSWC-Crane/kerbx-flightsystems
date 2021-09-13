use libkerbx::kerbx::Sheath_oneof_message::flightplan;
use libkerbx::kerbx::*;
use protobuf::{CodedInputStream, CodedOutputStream};
use std::error::Error;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::sync::broadcast::{Receiver, Sender};

pub struct PlanningServer {
    ip: String,
    port: String,
    listener: TcpListener,
}

impl PlanningServer {
    pub fn get_listener(&self) -> &TcpListener {
        &self.listener
    }
    fn empty_sheath() -> Option<Sheath> {
        let mut ret: Sheath = Sheath::new();
        ret.set_field_type(Sheath_MessageType::EMPTY);
        Some(ret)
    }

    /// Creates a new instance of the Planning Server on the provided ip and port
    pub async fn new(ip: String, port: String) -> Result<PlanningServer, Box<dyn Error>> {
        let listener = TcpListener::bind(format!("{}:{}", ip, port)).await?;

        Ok(PlanningServer { ip, port, listener })
    }

    /// Loop {} -- check all received packets, decode the sheats, and push them to a channel.
    pub async fn recv_and_decode(server: PlanningServer, tx: Sender<Sheath>) {
        loop {
            // Second Arg is a SocketAddr in case we wanted to implement an IP-based white list
            let (stream, _) = server
                .get_listener()
                .accept()
                .await
                .expect("recv_and_decode TCPStream Error.");

            let mut std_stream = stream
                .into_std()
                .expect("Error converting TCPStream to io stream.");
            let mut input = CodedInputStream::new(&mut std_stream);

            // TODO: Properly handle errors from read_message()
            // Getting hacky to ignore the error from read messsage. We will throw the error away
            // and create an empty sheath if the return is None. We should probably fix this later
            // as it will create a way for bugs to hide.
            let message: Sheath = input
                .read_message()
                .ok()
                .or_else(PlanningServer::empty_sheath)
                .unwrap();

            // Does not gurantee receivers will read the message on Ok()
            tx.send(message).expect("Error sending sheath to channel.");
        }
    }

    pub fn unsheath(item: Sheath) {
        match item.get_field_type() {
            Sheath_MessageType::COUNTCOWN => eprintln!("Countdown message!"),
            Sheath_MessageType::FLIGHTPLAN => eprintln!("Flightplan message!"),
            Sheath_MessageType::TELEMETRY => eprintln!("Telemetry message!"),
            Sheath_MessageType::WATCHDOG => eprintln!("Watchdog message!"),
            Sheath_MessageType::EMPTY => eprintln!("Empty sheath!"),
            _ => eprintln!("Unknown message type. Should be unreachable."),
        }
    }
}
