use crate::app::ServerCancellationToken;
use crate::app::client_address::ClientAddress;
use crate::app::handler::stream_quotes::{StreamQuotesContext, stream_quotes};
use crate::app::monitoring_router::MonitoringRouter;
use crate::app::tickers_router::{TickersRouter, TickersRouterError};
use crossbeam_channel::Sender;
use quote_streaming::{Commands, StockQuote};
use std::io::Read;
use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use tracing::{error, info, instrument};
use tracing_log::log::warn;

#[instrument(name = "Handle connection", skip_all)]
pub(super) fn handle_connection<R: Read>(
    mut reader: R,
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    tickers_router: Arc<TickersRouter>,
    monitoring_router: Arc<MonitoringRouter>,
    thread_tx: Sender<JoinHandle<Option<ClientAddress>>>,
) -> Option<ClientAddress> {
    let mut buffer = [0; 1024];
    let len = match reader.read(&mut buffer) {
        Ok(len) => len,
        Err(e) => {
            warn!("Failed to read from connection: {}", e);
            return None;
        }
    };
    let command = match rkyv::from_bytes::<Commands, rancor::Error>(&buffer[..len]) {
        Ok(command) => command,
        Err(e) => {
            warn!("Failed to deserialize command: {}", e);
            return None;
        }
    };
    info!("Received command: {:?}", command);

    match command {
        Commands::Stream {
            ticker,
            port,
            address,
        } => {
            let client_address = ClientAddress::new(address, port);
            let ctx = Arc::clone(&cancellation_token);
            let (quote_tx, quote_rx) = crossbeam_channel::unbounded::<StockQuote>();
            let (monitoring_tx, monitoring_rx) = crossbeam_channel::unbounded::<()>();

            if let Err(e) = tickers_router.add_routes(
                ticker
                    .iter()
                    .map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .as_slice(),
                quote_tx,
                client_address.clone(),
            ) {
                match e {
                    TickersRouterError::Unexpect => {
                        error!("Unexpected error from tickers router");
                        cancellation_token.cancel();
                    }
                    _ => {
                        warn!("Failed to add route: {}", e);
                    }
                }

                return Some(client_address);
            }

            if let Err(e) = monitoring_router.add_route(client_address.clone(), monitoring_tx) {
                warn!("Failed to add monitoring route: {}", e);
                cancellation_token.cancel();
                return Some(client_address);
            }

            let stream_quotes_ctx = StreamQuotesContext::new(
                ctx,
                udp_socket,
                quote_rx,
                monitoring_rx,
                client_address.clone(),
            );
            let thread = thread::spawn(move || stream_quotes(stream_quotes_ctx));

            if let Err(e) = thread_tx.send(thread) {
                warn!("Failed to send thread handle: {}", e);
                cancellation_token.cancel();
                return Some(client_address);
            };
        }
    }

    None
}
