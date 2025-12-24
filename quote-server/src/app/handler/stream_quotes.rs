use crate::app::ServerCancellationToken;
use chrono::Local;
use crossbeam_channel::Receiver;
use quote_streaming::StockQuote;
use std::net::{IpAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::{info, instrument, trace, warn};

#[instrument(name = "Stream quotes", skip(cancellation_token, udp_socket, quote_rx, monitoring_rx), fields(tickers = ?tickers, port, address))]
pub(super) fn stream_quotes(
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    quote_rx: Receiver<StockQuote>,
    monitoring_rx: Receiver<(IpAddr, u16)>,
    tickers: Vec<String>,
    port: u16,
    address: IpAddr,
) {
    let mut last_ping_time = Local::now().timestamp();
    loop {
        if cancellation_token.is_cancelled() {
            break;
        }

        if Local::now().timestamp() - last_ping_time > 5 {
            info!(
                "Last ping for {}:{} was more than 5 seconds ago, stop streaming quotes",
                address, port
            );
            break;
        }

        match quote_rx.try_recv() {
            Ok(quote) => {
                trace!("Received quotes");
                if tickers.iter().any(|ticker| ticker == quote.ticker()) {
                    let Ok(quotes_bytes) = rkyv::to_bytes::<rancor::Error>(&quote) else {
                        warn!("Failed to serialize quote: {:?}", quote);
                        cancellation_token.cancel();
                        break;
                    };
                    if let Err(e) = udp_socket.send_to(&quotes_bytes, (address, port)) {
                        warn!("Failed to send quote: {}", e);
                        break;
                    };
                }
            }
            Err(crossbeam_channel::TryRecvError::Empty) => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                warn!("Quotes receiver disconnected");
                cancellation_token.cancel();
                break;
            }
        }

        match monitoring_rx.try_recv() {
            Ok((ping_address, ping_port)) => {
                if address == ping_address && port == ping_port {
                    last_ping_time = Local::now().timestamp();
                }
            }
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                warn!("Quotes receiver disconnected");
                cancellation_token.cancel();
                break;
            }
            Err(crossbeam_channel::TryRecvError::Empty) => {}
        }
    }
}
