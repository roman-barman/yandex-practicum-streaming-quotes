use crate::app::ServerCancellationToken;
use crate::app::client_address::ClientAddress;
use crate::app::handler::connection_handler::{ConnectionHandlerContext, handle_connection};
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

pub(crate) struct AcceptConnectionContext {
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    tickers_router: Arc<TickersRouter>,
    monitoring_router: Arc<MonitoringRouter>,
    thread_tx: Sender<JoinHandle<Option<ClientAddress>>>,
}

impl AcceptConnectionContext {
    pub(crate) fn new(
        cancellation_token: Arc<ServerCancellationToken>,
        udp_socket: Arc<UdpSocket>,
        tickers_router: Arc<TickersRouter>,
        monitoring_router: Arc<MonitoringRouter>,
        thread_tx: Sender<JoinHandle<Option<ClientAddress>>>,
    ) -> Self {
        Self {
            cancellation_token,
            udp_socket,
            tickers_router,
            monitoring_router,
            thread_tx,
        }
    }
}

#[instrument(name = "Accept connection", skip_all)]
pub(crate) fn accept_connection(stream: TcpStream, context: AcceptConnectionContext) {
    let socket_addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(e) => {
            warn!("Failed to get peer address: {}", e);
            return;
        }
    };

    let handler_context = ConnectionHandlerContext::new(
        Arc::clone(&context.cancellation_token),
        context.udp_socket,
        context.tickers_router,
        context.monitoring_router,
        context.thread_tx.clone(),
    );
    let thread = thread::spawn(move || {
        info!("Accepted connection from {}", socket_addr);
        let result = handle_connection(stream, handler_context);
        info!("Connection closed for {}", socket_addr);
        result
    });
    if let Err(e) = context.thread_tx.send(thread) {
        warn!("Failed to send thread handle: {}", e);
        context.cancellation_token.cancel();
    }
}
