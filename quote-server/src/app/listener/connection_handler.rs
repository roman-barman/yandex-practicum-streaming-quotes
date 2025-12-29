use crate::app::ServerCancellationToken;
use crate::app::client_address::ClientAddress;
use crate::app::monitoring::{MonitoringRouter, MonitoringRouterError};
use crate::app::quote_streaming::{
    StreamQuotesContext, TickersRouter, TickersRouterError, stream_quotes,
};
use crossbeam_channel::Sender;
use quote_streaming::{Request, Response, StockQuote};
use std::io::{Read, Write};
use std::net::UdpSocket;
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
            send_response(&mut stream, response, &context.cancellation_token);
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
        } => {
            let client_address = ClientAddress::new(address, port);
            let cancellation_token = Arc::clone(&context.cancellation_token);
            let result = start_stream_quotes(client_address.clone(), ticker, context);
            if let Err(e) = result {
                error!("Failed to start stream quotes: {}", e);
                cancellation_token.cancel();
                return Some(client_address);
            }
            let response = Response::Ok;
            send_response(&mut stream, response, &cancellation_token);
            None
        }
        Request::Ping => {
            let response = Response::Error("Unexpected request".to_string());
            send_response(&mut stream, response, &context.cancellation_token);
            None
        }
    }
}

fn start_stream_quotes(
    client_address: ClientAddress,
    tickers: Vec<String>,
    context: ConnectionHandlerContext,
) -> Result<(), StreamQuotesError> {
    let (quote_tx, quote_rx) = crossbeam_channel::unbounded::<StockQuote>();
    let (monitoring_tx, monitoring_rx) = crossbeam_channel::unbounded::<()>();

    context
        .tickers_router
        .add_routes(tickers, quote_tx, client_address.clone())?;
    context
        .monitoring_router
        .add_route(client_address.clone(), monitoring_tx)?;

    let stream_quotes_ctx = StreamQuotesContext::new(
        context.cancellation_token,
        context.udp_socket,
        quote_rx,
        monitoring_rx,
        client_address.clone(),
    );
    let thread = thread::spawn(move || stream_quotes(stream_quotes_ctx));
    context
        .thread_tx
        .send(thread)
        .map_err(|e| StreamQuotesError::SendChannel(e.to_string()))?;

    Ok(())
}

fn send_response<W: Write>(
    writer: &mut W,
    response: Response,
    cancellation_token: &Arc<ServerCancellationToken>,
) {
    let response: Result<Vec<u8>, _> = response.try_into();
    let response = match response {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to serialize response: {}", e);
            cancellation_token.cancel();
            return;
        }
    };

    if let Err(e) = writer.write_all(&response) {
        warn!("Failed to send response: {}", e);
    }
}

fn read_request<R: Read>(mut reader: R) -> Result<Request, ReadRequestError> {
    let mut buffer = [0; 1024];
    let len = reader.read(&mut buffer)?;
    let command = Request::try_from(&buffer[..len])?;
    Ok(command)
}

#[derive(Debug, thiserror::Error)]
enum ReadRequestError {
    #[error("Failed to read request: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to deserialize request: {0}")]
    InvalidRequest(#[from] rancor::Error),
}

#[derive(Debug, thiserror::Error)]
enum StreamQuotesError {
    #[error("Failed to add ticker route: {0}")]
    TickersRouter(#[from] TickersRouterError),
    #[error("Failed to add monitoring route: {0}")]
    MonitoringRouter(#[from] MonitoringRouterError),
    #[error("Failed to send thread message: {0}")]
    SendChannel(String),
}
