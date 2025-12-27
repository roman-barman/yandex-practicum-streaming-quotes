use crate::app::ServerCancellationToken;
use crate::app::client_address::ClientAddress;
use crate::app::handler::stream_quotes::{StreamQuotesContext, stream_quotes};
use crate::app::monitoring_router::MonitoringRouter;
use crate::app::tickers_router::TickersRouter;
use crossbeam_channel::Sender;
use quote_streaming::{Request, Response, StockQuote};
use std::io::{Read, Write};
use std::net::{IpAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use tracing::{error, info, instrument};
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
pub(super) fn handle_connection<R: Read + Write>(
    mut stream: R,
    context: ConnectionHandlerContext,
) -> Option<ClientAddress> {
    let request = match read_request(&mut stream) {
        Ok(request) => {
            info!("Received request: {:?}", request);
            request
        }
        Err(ReadRequestError::InvalidRequest(e)) => {
            warn!("Invalid request: {}", e);
            let response = Response::Error("Invalid request".to_string());
            send_response_safe(&mut stream, &response, &context);
            return None;
        }
        Err(e) => {
            warn!("Failed to read request: {}", e);
            return None;
        }
    };

    match request {
        Request::StreamTickers {
            ticker,
            port,
            address,
        } => start_stream_quotes(address, port, ticker, context),
        Request::Ping => {
            let response = Response::Error("Unexpected request".to_string());
            send_response_safe(&mut stream, &response, &context);
            None
        }
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

fn send_response_safe<W: Write>(
    writer: &mut W,
    response: &Response,
    context: &ConnectionHandlerContext,
) {
    if let Err(e) = write_response(response, writer) {
        match e {
            WriteResponseError::Io(e) => warn!("Failed to write response: {}", e),
            WriteResponseError::InvalidResponse(e) => {
                error!("Failed to serialize response: {}", e);
                context.cancellation_token.cancel();
            }
        }
    }
}

fn read_request<R: Read>(mut reader: R) -> Result<Request, ReadRequestError> {
    let mut buffer = [0; 1024];
    let len = reader.read(&mut buffer)?;
    let command = rkyv::from_bytes::<Request, rancor::Error>(&buffer[..len])?;
    Ok(command)
}

fn write_response<W: Write>(response: &Response, writer: &mut W) -> Result<(), WriteResponseError> {
    let response = rkyv::to_bytes::<rancor::Error>(response)?;
    writer.write_all(&response)?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum ReadRequestError {
    #[error("Failed to read request: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to deserialize request: {0}")]
    InvalidRequest(#[from] rancor::Error),
}

#[derive(Debug, thiserror::Error)]
enum WriteResponseError {
    #[error("Failed to write response: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to serialize response: {0}")]
    InvalidResponse(#[from] rancor::Error),
}
