use crate::app::ServerCancellationToken;
use crate::app::handler::connection_handler::handle_connection;
use crossbeam_channel::Receiver;
use quote_streaming::StockQuote;
use std::net::{TcpStream, UdpSocket};
use std::sync::Arc;
use std::thread;
use tracing::{info, instrument, warn};

mod connection_handler;
mod error;
mod stream_quotes;

#[instrument(name = "Accept connection", skip_all)]
pub(crate) fn accept_connection(
    stream: TcpStream,
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    rx: Receiver<StockQuote>,
) -> thread::JoinHandle<()> {
    thread::spawn(|| {
        let socket_addr = match stream.peer_addr() {
            Ok(addr) => addr,
            Err(e) => {
                warn!("Failed to get peer address: {}", e);
                return;
            }
        };
        info!("Accepted connection from {}", socket_addr);

        match handle_connection(stream, cancellation_token, udp_socket, rx) {
            Ok(()) => info!("Connection closed for {}", socket_addr),
            Err(e) => {
                warn!("{}", e);
            }
        }
    })
}
