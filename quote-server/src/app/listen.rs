use crate::app::client_address::ClientAddress;
use crate::app::handler::accept_connection;
use crate::app::monitoring_router::MonitoringRouter;
use crate::app::server_cancellation_token::ServerCancellationToken;
use crate::app::tickers_router::TickersRouter;
use crossbeam_channel::{Receiver, Sender};
use std::io::ErrorKind;
use std::net::{TcpListener, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use tracing::{error, info, instrument, warn};

const POLL_INTERVAL: Duration = Duration::from_millis(100);

#[instrument(name = "Run listening", skip_all)]
pub(crate) fn run_listening(
    tcp_listener: TcpListener,
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    tickers_router: Arc<TickersRouter>,
    monitoring_router: Arc<MonitoringRouter>,
) -> (Receiver<JoinHandle<Option<ClientAddress>>>, JoinHandle<()>) {
    let (tx, rx) = crossbeam_channel::unbounded::<JoinHandle<Option<ClientAddress>>>();
    let thread = thread::spawn(move || {
        listen(
            tcp_listener,
            cancellation_token,
            udp_socket,
            tickers_router,
            monitoring_router,
            tx,
        )
    });
    (rx, thread)
}

#[instrument(name = "Listen", skip_all)]
fn listen(
    tcp_listener: TcpListener,
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    tickers_router: Arc<TickersRouter>,
    monitoring_router: Arc<MonitoringRouter>,
    thread_tx: Sender<JoinHandle<Option<ClientAddress>>>,
) {
    let Ok(local_addr) = tcp_listener.local_addr() else {
        error!("Failed to get local address");
        cancellation_token.cancel();
        return;
    };

    info!("Listening on {}:{}", local_addr.ip(), local_addr.port());
    for stream in tcp_listener.incoming() {
        if cancellation_token.is_cancelled() {
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

        accept_connection(
            stream,
            Arc::clone(&cancellation_token),
            Arc::clone(&udp_socket),
            Arc::clone(&tickers_router),
            Arc::clone(&monitoring_router),
            thread_tx.clone(),
        );
    }
}
