use crate::app::ServerCancellationToken;
use crate::app::handler::error::HandlerError;
use crate::app::handler::stream_quotes::stream_quotes;
use crossbeam_channel::Receiver;
use quote_streaming::{Commands, StockQuote};
use std::io::Read;
use std::net::UdpSocket;
use std::sync::Arc;
use tracing::{info, instrument};

#[instrument(name = "Handle connection", skip_all)]
pub(super) fn handle_connection<R: Read>(
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
