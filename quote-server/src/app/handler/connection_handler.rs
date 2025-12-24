use crate::app::ServerCancellationToken;
use crate::app::handler::stream_quotes::stream_quotes;
use crossbeam_channel::{Receiver, Sender};
use quote_streaming::{Commands, StockQuote};
use std::io::Read;
use std::net::{IpAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use tracing::{info, instrument};
use tracing_log::log::warn;

#[instrument(name = "Handle connection", skip_all)]
pub(super) fn handle_connection<R: Read>(
    mut reader: R,
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    quote_rx: Receiver<StockQuote>,
    monitoring_rx: Receiver<(IpAddr, u16)>,
    thread_tx: Sender<JoinHandle<()>>,
) {
    let mut buffer = [0; 1024];
    let Ok(len) = reader.read(&mut buffer) else {
        warn!("Failed to read from connection");
        return;
    };

    let Ok(command) = rkyv::from_bytes::<Commands, rancor::Error>(&buffer[..len]) else {
        warn!("Failed to deserialize command");
        return;
    };
    info!("Received command: {:?}", command);

    match command {
        Commands::Stream {
            ticker,
            port,
            address,
        } => {
            let ctx = Arc::clone(&cancellation_token);
            let thread = thread::spawn(move || {
                stream_quotes(
                    ctx,
                    udp_socket,
                    quote_rx,
                    monitoring_rx,
                    ticker,
                    port,
                    address,
                )
            });
            if let Err(e) = thread_tx.send(thread) {
                warn!("Failed to send thread handle: {}", e);
                cancellation_token.cancel();
            }
        }
    }
}
