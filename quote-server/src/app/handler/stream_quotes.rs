use crate::app::ServerCancellationToken;
use crate::app::client_address::ClientAddress;
use crossbeam_channel::{Receiver, select_biased};
use quote_streaming::StockQuote;
use std::net::UdpSocket;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, instrument, trace, warn};

#[instrument(
    name = "Stream quotes",
    skip(cancellation_token, udp_socket, quote_rx, monitoring_rx),
    fields(address)
)]
pub(super) fn stream_quotes(
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    quote_rx: Receiver<StockQuote>,
    monitoring_rx: Receiver<()>,
    address: ClientAddress,
) -> Option<ClientAddress> {
    let mut last_ping_time = Instant::now();
    let timeout_duration = Duration::from_secs(5);

    loop {
        select_biased! {
            recv(monitoring_rx) -> msg => {
                match msg {
                    Ok(_) => {
                        last_ping_time = Instant::now();
                    }
                    Err(_) => {
                        warn!("Monitoring receiver disconnected");
                        cancellation_token.cancel();
                        break;
                    }
                }
            }
            recv(quote_rx) -> msg => {
                match msg {
                    Ok(quote) => {
                        trace!("Received quotes {}", quote.ticker());
                        let Ok(quotes_bytes) = rkyv::to_bytes::<rancor::Error>(&quote) else {
                                warn!("Failed to serialize quote: {:?}", quote);
                                cancellation_token.cancel();
                                break;
                            };
                            if let Err(e) = udp_socket.send_to(&quotes_bytes, address.address()) {
                                warn!("Failed to send quote: {}", e);
                                break;
                            };
                    },
                    Err(_) => {
                        warn!("Quotes receiver disconnected");
                        cancellation_token.cancel();
                        break;
                    }
                }
            }
            default(Duration::from_millis(100)) => {
                if cancellation_token.is_cancelled() {
                    break;
                }
                if last_ping_time.elapsed() > timeout_duration {
                    info!(
                        "Last ping for {} was more than 5 seconds ago, stop streaming quotes",
                        address
                    );
                    break;
                }
            }
        }
    }

    Some(address)
}
