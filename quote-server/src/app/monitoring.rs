use crate::app::ServerCancellationToken;
use crossbeam_channel::{Receiver, Sender, unbounded};
use quote_streaming::KeepAlive;
use std::io::ErrorKind;
use std::net::{IpAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use tracing::{error, instrument, trace, warn};

#[instrument(name = "Run monitoring", skip_all)]
pub(crate) fn run_monitoring(
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
) -> (Receiver<(IpAddr, u16)>, JoinHandle<()>) {
    let (tx, rx) = unbounded::<(IpAddr, u16)>();
    let thread = thread::spawn(move || monitoring(cancellation_token, udp_socket, tx));
    (rx, thread)
}

#[instrument(name = "Monitoring", skip_all)]
fn monitoring(
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    tx: Sender<(IpAddr, u16)>,
) {
    let pong = match rkyv::to_bytes::<rancor::Error>(&KeepAlive::Pong) {
        Ok(bytes) => bytes,
        Err(err) => {
            error!("Failed to serialize pong message: {}", err);
            return;
        }
    };

    let mut buffer = [0; 1024];
    loop {
        if cancellation_token.is_cancelled() {
            break;
        }

        match udp_socket.recv_from(&mut buffer) {
            Ok((size, address)) => {
                let keep_alive = rkyv::from_bytes::<KeepAlive, rancor::Error>(&buffer[..size]);
                match keep_alive {
                    Ok(KeepAlive::Ping) => {
                        trace!("Received ping from {}", address);
                        match udp_socket.send_to(&pong, address) {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Failed to send pong to {}: {}", address, e);
                                continue;
                            }
                        }

                        let ip_address = address.ip();
                        let port = address.port();
                        match tx.send((ip_address, port)) {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Failed to send ping address to channel: {}", e);
                                cancellation_token.cancel();
                                return;
                            }
                        }
                    }
                    Ok(KeepAlive::Pong) => warn!("Received pong from {}", address),
                    Err(e) => warn!("Failed to deserialize keep alive message: {}", e),
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                continue;
            }
            Err(e) => error!("Failed to receive keep alive message: {}", e),
        }
    }
}
