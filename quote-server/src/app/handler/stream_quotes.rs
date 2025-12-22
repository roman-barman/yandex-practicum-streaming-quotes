use crate::app::ServerCancellationToken;
use crate::app::handler::error::HandlerError;
use crossbeam_channel::Receiver;
use quote_streaming::StockQuote;
use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::{instrument, trace, warn};

#[instrument(name = "Stream quotes", skip(cancellation_token, udp_socket, rx), fields(tickers = ?tickers, port, address))]
pub(super) fn stream_quotes(
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
