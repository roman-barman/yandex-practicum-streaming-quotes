mod connection_handler;
mod handler;

use crate::app::client_address::ClientAddress;
use crate::app::listener::handler::{AcceptConnectionContext, accept_connection};
use crate::app::monitoring::MonitoringRouter;
use crate::app::quote_streaming::TickersRouter;
use crate::app::server_cancellation_token::ServerCancellationToken;
use crossbeam_channel::{Receiver, Sender};
use std::io::ErrorKind;
use std::net::{TcpListener, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use tracing::{error, info, instrument, warn};

const POLL_INTERVAL: Duration = Duration::from_millis(100);

pub(crate) struct ListenContext {
    tcp_listener: TcpListener,
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    tickers_router: Arc<TickersRouter>,
    monitoring_router: Arc<MonitoringRouter>,
}

impl ListenContext {
    pub(crate) fn new(
        tcp_listener: TcpListener,
        cancellation_token: Arc<ServerCancellationToken>,
        udp_socket: Arc<UdpSocket>,
        tickers_router: Arc<TickersRouter>,
        monitoring_router: Arc<MonitoringRouter>,
    ) -> Self {
        Self {
            tcp_listener,
            cancellation_token,
            udp_socket,
            tickers_router,
            monitoring_router,
        }
    }

    fn to_accept_context(
        &self,
        thread_tx: Sender<JoinHandle<Option<ClientAddress>>>,
    ) -> AcceptConnectionContext {
        AcceptConnectionContext::new(
            Arc::clone(&self.cancellation_token),
            Arc::clone(&self.udp_socket),
            Arc::clone(&self.tickers_router),
            Arc::clone(&self.monitoring_router),
            thread_tx,
        )
    }
}

#[instrument(name = "Run listening", skip_all)]
pub(crate) fn run_listening(
    context: ListenContext,
) -> (Receiver<JoinHandle<Option<ClientAddress>>>, JoinHandle<()>) {
    let (tx, rx) = crossbeam_channel::unbounded::<JoinHandle<Option<ClientAddress>>>();
    let thread = thread::spawn(move || listen(context, tx));
    (rx, thread)
}

#[instrument(name = "Listen", skip_all)]
fn listen(context: ListenContext, thread_tx: Sender<JoinHandle<Option<ClientAddress>>>) {
    let Ok(local_addr) = context.tcp_listener.local_addr() else {
        error!("Failed to get local address");
        context.cancellation_token.cancel();
        return;
    };

    info!("Listening on {}:{}", local_addr.ip(), local_addr.port());
    for stream in context.tcp_listener.incoming() {
        if context.cancellation_token.is_cancelled() {
            break;
        }

        let stream = match stream {
            Ok(stream) => stream,
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                thread::sleep(POLL_INTERVAL);
                continue;
            }
            Err(e) => {
                warn!("Failed to accept connection: {}", e);
                continue;
            }
        };

        accept_connection(stream, context.to_accept_context(thread_tx.clone()));
    }
}
