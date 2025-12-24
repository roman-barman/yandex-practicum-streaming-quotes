use crate::app::ServerCancellationToken;
use crate::app::handler::connection_handler::handle_connection;
use crossbeam_channel::{Receiver, Sender};
use quote_streaming::StockQuote;
use std::net::{IpAddr, TcpStream, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use tracing::{info, instrument, warn};

mod connection_handler;
mod stream_quotes;

#[instrument(name = "Accept connection", skip_all)]
pub(crate) fn accept_connection(
    stream: TcpStream,
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    quote_rx: Receiver<StockQuote>,
    monitoring_rx: Receiver<(IpAddr, u16)>,
    thread_tx: Sender<JoinHandle<()>>,
) {
    let thread_tx_copy = thread_tx.clone();
    let ctx = Arc::clone(&cancellation_token);
    let thread = thread::spawn(|| {
        let socket_addr = match stream.peer_addr() {
            Ok(addr) => addr,
            Err(e) => {
                warn!("Failed to get peer address: {}", e);
                return;
            }
        };
        info!("Accepted connection from {}", socket_addr);

        handle_connection(
            stream,
            ctx,
            udp_socket,
            quote_rx,
            monitoring_rx,
            thread_tx_copy,
        );
        info!("Connection closed for {}", socket_addr)
    });
    if let Err(e) = thread_tx.send(thread) {
        warn!("Failed to send thread handle: {}", e);
        cancellation_token.cancel();
    }
}
