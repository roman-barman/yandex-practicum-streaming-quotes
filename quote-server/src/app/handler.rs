use crate::app::ServerCancellationToken;
use crate::app::client_address::ClientAddress;
use crate::app::handler::connection_handler::handle_connection;
use crate::app::monitoring_router::MonitoringRouter;
use crate::app::tickers_router::TickersRouter;
use crossbeam_channel::Sender;
use std::net::{TcpStream, UdpSocket};
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
    tickers_router: Arc<TickersRouter>,
    monitoring_router: Arc<MonitoringRouter>,
    thread_tx: Sender<JoinHandle<Option<ClientAddress>>>,
) {
    let thread_tx_copy = thread_tx.clone();
    let ctx = Arc::clone(&cancellation_token);
    let socket_addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(e) => {
            warn!("Failed to get peer address: {}", e);
            return;
        }
    };
    let thread = thread::spawn(move || {
        info!("Accepted connection from {}", socket_addr);
        let result = handle_connection(
            stream,
            ctx,
            udp_socket,
            tickers_router,
            monitoring_router,
            thread_tx_copy,
        );
        info!("Connection closed for {}", socket_addr);
        result
    });
    if let Err(e) = thread_tx.send(thread) {
        warn!("Failed to send thread handle: {}", e);
        cancellation_token.cancel();
    }
}
