use crate::app::ServerCancellationToken;
use crate::app::client_address::ClientAddress;
use crate::app::handler::stream_quotes::{StreamQuotesContext, stream_quotes};
use crate::app::monitoring_router::MonitoringRouter;
use crate::app::tickers_router::TickersRouter;
use crossbeam_channel::Sender;
use quote_streaming::{Commands, StockQuote};
use std::io::Read;
use std::net::{IpAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use tracing::{info, instrument};
use tracing_log::log::warn;

pub(crate) struct ConnectionHandlerContext {
    cancellation_token: Arc<ServerCancellationToken>,
    udp_socket: Arc<UdpSocket>,
    tickers_router: Arc<TickersRouter>,
    monitoring_router: Arc<MonitoringRouter>,
    thread_tx: Sender<JoinHandle<Option<ClientAddress>>>,
}

impl ConnectionHandlerContext {
    pub(crate) fn new(
        cancellation_token: Arc<ServerCancellationToken>,
        udp_socket: Arc<UdpSocket>,
        tickers_router: Arc<TickersRouter>,
        monitoring_router: Arc<MonitoringRouter>,
        thread_tx: Sender<JoinHandle<Option<ClientAddress>>>,
    ) -> Self {
        Self {
            cancellation_token,
            udp_socket,
            tickers_router,
            monitoring_router,
            thread_tx,
        }
    }
}

#[instrument(name = "Handle connection", skip_all)]
pub(super) fn handle_connection<R: Read>(
    mut reader: R,
    context: ConnectionHandlerContext,
) -> Option<ClientAddress> {
    let command = match read_command(&mut reader) {
        Ok(command) => {
            info!("Received command: {:?}", command);
            command
        }
        Err(e) => {
            warn!("Failed to read command: {}", e);
            return None;
        }
    };

    match command {
        Commands::Stream {
            ticker,
            port,
            address,
        } => start_stream_quotes(address, port, ticker, context),
    }
}

fn start_stream_quotes(
    address: IpAddr,
    port: u16,
    tickers: Vec<String>,
    context: ConnectionHandlerContext,
) -> Option<ClientAddress> {
    let client_address = ClientAddress::new(address, port);
    let (quote_tx, quote_rx) = crossbeam_channel::unbounded::<StockQuote>();
    let (monitoring_tx, monitoring_rx) = crossbeam_channel::unbounded::<()>();

    if let Err(e) = context
        .tickers_router
        .add_routes(tickers, quote_tx, client_address.clone())
    {
        warn!("Failed to add route: {}", e);
        return Some(client_address);
    }

    if let Err(e) = context
        .monitoring_router
        .add_route(client_address.clone(), monitoring_tx)
    {
        warn!("Failed to add monitoring route: {}", e);
        context.cancellation_token.cancel();
        return Some(client_address);
    }

    let stream_quotes_ctx = StreamQuotesContext::new(
        Arc::clone(&context.cancellation_token),
        context.udp_socket,
        quote_rx,
        monitoring_rx,
        client_address.clone(),
    );
    let thread = thread::spawn(move || stream_quotes(stream_quotes_ctx));

    if let Err(e) = context.thread_tx.send(thread) {
        warn!("Failed to send thread handle: {}", e);
        context.cancellation_token.cancel();
        return Some(client_address);
    };

    None
}

fn read_command<R: Read>(mut reader: R) -> Result<Commands, Box<dyn std::error::Error>> {
    let mut buffer = [0; 1024];
    let len = reader.read(&mut buffer)?;
    let command = rkyv::from_bytes::<Commands, rancor::Error>(&buffer[..len])?;
    Ok(command)
}
