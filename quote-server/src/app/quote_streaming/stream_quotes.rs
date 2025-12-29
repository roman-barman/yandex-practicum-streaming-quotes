use crate::app::ServerCancellationToken;
use crate::app::client_address::ClientAddress;
use crossbeam_channel::{Receiver, select_biased};
use quote_streaming::{Response, StockQuote};
use std::net::UdpSocket;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, instrument, trace, warn};

const TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) struct StreamQuotesContext {
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    quote_rx: Receiver<StockQuote>,
    monitoring_rx: Receiver<()>,
    address: ClientAddress,
}

impl StreamQuotesContext {
    pub(crate) fn new(
        cancellation_token: Arc<ServerCancellationToken>,
        udp_socket: Arc<UdpSocket>,
        quote_rx: Receiver<StockQuote>,
        monitoring_rx: Receiver<()>,
        address: ClientAddress,
    ) -> Self {
        Self {
            cancellation_token,
            udp_socket,
            quote_rx,
            monitoring_rx,
            address,
        }
    }
}

#[instrument(name = "Stream quotes", skip_all)]
pub(crate) fn stream_quotes(context: StreamQuotesContext) -> Option<ClientAddress> {
    let mut last_ping_time = Instant::now();

    while !context.cancellation_token.is_cancelled() {
        select_biased! {
            recv(context.monitoring_rx) -> msg => {
                match msg {
                    Ok(_) => {
                        last_ping_time = Instant::now();
                    }
                    Err(_) => {
                        warn!("Monitoring receiver disconnected");
                        context.cancellation_token.cancel();
                        break;
                    }
                }
            }
            recv(context.quote_rx) -> msg => {
                match msg {
                    Ok(quote) => {
                        trace!("Received quotes {}", quote.ticker());
                        let response: Result<Vec<u8>,_> = Response::Quote(quote.clone()).try_into();
                        let Ok(quotes_bytes) = response else {
                                warn!("Failed to serialize quote: {:?}", quote);
                                context.cancellation_token.cancel();
                                break;
                            };
                            if let Err(e) = context.udp_socket.send_to(&quotes_bytes, context.address.address()) {
                                warn!("Failed to send quote: {}", e);
                                break;
                            };
                    },
                    Err(_) => {
                        warn!("Quotes receiver disconnected");
                        context.cancellation_token.cancel();
                        break;
                    }
                }
            }
            default(Duration::from_millis(100)) => {
                if last_ping_time.elapsed() > TIMEOUT {
                    info!(
                        "Last ping for {} was more than 5 seconds ago, stop streaming quotes",
                        context.address
                    );
                    break;
                }
            }
        }
    }

    Some(context.address)
}
