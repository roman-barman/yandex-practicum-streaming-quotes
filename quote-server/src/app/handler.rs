use crate::app::server_cancellation_token::ServerCancellationToken;
use crossbeam_channel::Receiver;
use quote_streaming::{Commands, StockQuote};
use std::io::Read;
use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::{info, instrument, trace, warn};

#[instrument(name = "Handle connection", skip_all)]
pub(crate) fn handle_connection<R: Read>(
    mut reader: R,
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    rx: Receiver<StockQuote>,
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
        } => stream_quotes(cancellation_token, udp_socket, rx, ticker, port, address)?,
    }

    Ok(())
}

#[instrument(name = "Stream quotes", skip(cancellation_token, udp_socket, rx), fields(tickers = ?tickers, port, address))]
fn stream_quotes(
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    rx: Receiver<StockQuote>,
    tickers: Vec<String>,
    port: u16,
    address: std::net::IpAddr,
) -> Result<(), HandlerError> {
    loop {
        if cancellation_token.is_cancelled() {
            return Ok(());
        }
        match rx.try_recv() {
            Ok(quote) => {
                trace!("Received quotes");
                if tickers.iter().any(|ticker| ticker == quote.ticker()) {
                    let quotes_bytes = rkyv::to_bytes::<rancor::Error>(&quote)?;
                    udp_socket.send_to(&quotes_bytes, (address, port))?;
                }
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
