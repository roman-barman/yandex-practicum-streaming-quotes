use crossbeam_channel::Receiver;
use quote_streaming::{Commands, StockQuote};
use std::io::Read;
use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;
use tracing::{info, instrument, trace, warn};

#[instrument(name = "Handle connection", skip_all)]
pub(crate) fn handle_connection<R: Read>(
    mut reader: R,
    is_server_working: Arc<AtomicBool>,
    udp_socket: Arc<UdpSocket>,
    rx: Receiver<Vec<StockQuote>>,
) -> Result<(), HandlerError> {
    let mut buffer = [0; 1024];
    let len = reader.read(&mut buffer)?;

    let command = rkyv::from_bytes::<Commands, rancor::Error>(&buffer[..len])?;
    info!("Received command: {:?}", command);

    match command {
        Commands::Stream {
            ticker,
            port,
            address,
        } => stream_quotes(is_server_working, udp_socket, rx, ticker, port, address)?,
    }

    Ok(())
}

#[instrument(name = "Stream quotes", skip(is_server_working, udp_socket, rx), fields(tickers = ?tickers, port, address))]
fn stream_quotes(
    is_server_working: Arc<AtomicBool>,
    udp_socket: Arc<UdpSocket>,
    rx: Receiver<Vec<StockQuote>>,
    tickers: Vec<String>,
    port: u16,
    address: std::net::IpAddr,
) -> Result<(), HandlerError> {
    loop {
        if !is_server_working.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }
        match rx.try_recv() {
            Ok(quotes) => {
                trace!("Received quotes");
                let quotes_to_send = quotes
                    .into_iter()
                    .filter(|quote| tickers.iter().any(|ticker| ticker == quote.ticker()))
                    .collect::<Vec<_>>();
                let quotes_bytes = rkyv::to_bytes::<rancor::Error>(&quotes_to_send)?;
                udp_socket.send_to(&quotes_bytes, (address, port))?;
            }
            Err(crossbeam_channel::TryRecvError::Empty) => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                warn!("Quotes receiver disconnected");
                return Ok(());
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum HandlerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Deserialization command error: {0}")]
    DeserializationOrSerialization(#[from] rancor::Error),
}
