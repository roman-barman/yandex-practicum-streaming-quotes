use crate::app::ServerCancellationToken;
use crate::app::client_address::ClientAddress;
use crate::app::monitoring_router::MonitoringRouter;
use quote_streaming::KeepAlive;
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use tracing::{error, instrument, trace, warn};

#[instrument(name = "Run monitoring", skip_all)]
pub(crate) fn run_monitoring(
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    router: Arc<MonitoringRouter>,
) -> JoinHandle<()> {
    thread::spawn(move || monitoring(cancellation_token, udp_socket, router))
}

#[instrument(name = "Monitoring", skip_all)]
fn monitoring(
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    router: Arc<MonitoringRouter>,
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
                        let address = ClientAddress::new(ip_address, port);
                        if let Err(e) = router.send_ping(&address) {
                            warn!("Failed to send ping to monitoring router: {}", e);
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
