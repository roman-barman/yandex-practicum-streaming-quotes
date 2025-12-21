use crate::app::ServerCancellationToken;
use quote_streaming::KeepAlive;
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::Arc;
use std::time::Duration;
use tracing::instrument;

#[instrument(name = "Start monitoring", skip_all)]
pub(crate) fn start_monitoring(
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
) {
    let mut buffer = [0; 1024];
    loop {
        if cancellation_token.is_cancelled() {
            break;
        }

        match udp_socket.recv_from(&mut buffer) {
            Ok((size, address)) => {
                let keep_alive = rkyv::from_bytes::<KeepAlive, rancor::Error>(&buffer[..size]);
                match keep_alive {
                    Ok(KeepAlive::Ping) => tracing::trace!("Received ping from {}", address),
                    Ok(KeepAlive::Pong) => tracing::warn!("Received pong from {}", address),
                    Err(e) => tracing::warn!("Failed to deserialize keep alive message: {}", e),
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(500));
            }
            Err(e) => tracing::error!("Failed to receive keep alive message: {}", e),
        }
    }
}
