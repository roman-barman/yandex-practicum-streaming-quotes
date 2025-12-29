use crate::app::cancellation_token::CancellationToken;
use quote_streaming::Request;
use std::io::ErrorKind;
use std::net::{IpAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const PING_INTERVAL: Duration = Duration::from_secs(2);

pub(super) fn start_ping(
    cancellation_token: Arc<CancellationToken>,
    socket: Arc<UdpSocket>,
    sender_address: IpAddr,
    server_port: u16,
) -> thread::JoinHandle<()> {
    thread::spawn(move || ping(cancellation_token, socket, sender_address, server_port))
}

fn ping(
    cancellation_token: Arc<CancellationToken>,
    socket: Arc<UdpSocket>,
    sender_address: IpAddr,
    server_port: u16,
) {
    let ping: Result<Vec<u8>, _> = Request::Ping.try_into();
    let ping = match ping {
        Ok(ping) => ping,
        Err(e) => {
            eprintln!("Failed to serialize ping request: {}", e);
            cancellation_token.cancel();
            return;
        }
    };

    while !cancellation_token.is_cancelled() {
        if let Err(e) = socket.send_to(&ping, (sender_address, server_port)) {
            if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock {
                eprintln!("Server disconnected");
            } else {
                eprintln!("Failed to send ping: {}", e);
            }
            cancellation_token.cancel();
            break;
        }
        thread::sleep(PING_INTERVAL);
    }
}
