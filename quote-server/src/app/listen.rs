use crate::app::handler::accept_connection;
use crate::app::server_cancellation_token::ServerCancellationToken;
use crossbeam_channel::{Receiver, Sender};
use quote_streaming::StockQuote;
use std::io::ErrorKind;
use std::net::{IpAddr, TcpListener, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use tracing::{error, info, instrument};

#[instrument(name = "Run listening", skip_all)]
pub(crate) fn run_listening(
    tcp_listener: TcpListener,
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    quote_rx: Receiver<StockQuote>,
    monitoring_rx: Receiver<(IpAddr, u16)>,
) -> (Receiver<JoinHandle<()>>, JoinHandle<()>) {
    let (tx, rx) = crossbeam_channel::unbounded::<JoinHandle<()>>();
    let thread = thread::spawn(move || {
        listen(
            tcp_listener,
            cancellation_token,
            udp_socket,
            quote_rx,
            monitoring_rx,
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
    quote_rx: Receiver<StockQuote>,
    monitoring_rx: Receiver<(IpAddr, u16)>,
    thread_tx: Sender<JoinHandle<()>>,
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
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
                break;
            }
        };

        accept_connection(
            stream,
            Arc::clone(&cancellation_token),
            Arc::clone(&udp_socket),
            quote_rx.clone(),
            monitoring_rx.clone(),
            thread_tx.clone(),
        );
    }
}
