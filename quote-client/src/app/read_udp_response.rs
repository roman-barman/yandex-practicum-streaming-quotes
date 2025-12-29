use crate::app::cancellation_token::CancellationToken;
use quote_streaming::Response;
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::Arc;

const MAX_ATTEMPTS: usize = 10;

pub(crate) fn read_udp_response(
    cancellation_token: Arc<CancellationToken>,
    socket: Arc<UdpSocket>,
) -> Result<(), ReadUdpResponseError> {
    let mut buffer = [0; 1024];
    let mut attempts = 0;

    while !cancellation_token.is_cancelled() {
        let len = match socket.recv(&mut buffer) {
            Ok(read_bytes) => read_bytes,
            Err(e) => {
                let error = if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock
                {
                    ReadUdpResponseError::ServerDisconnected
                } else {
                    ReadUdpResponseError::Io(e)
                };
                cancellation_token.cancel();
                return Err(error);
            }
        };
        match Response::try_from(&buffer[..len]) {
            Ok(response) => {
                attempts = 0;
                match response {
                    Response::Quote(quote) => println!("{}", quote),
                    Response::Pong | Response::Ok => {}
                    Response::Error(err) => println!("Server send error: {}", err),
                }
            }
            Err(e) => {
                if attempts < MAX_ATTEMPTS {
                    attempts += 1;
                } else {
                    cancellation_token.cancel();
                    return Err(ReadUdpResponseError::InvalidResponse(e));
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ReadUdpResponseError {
    #[error("Server disconnected")]
    ServerDisconnected,
    #[error("Failed to read UDP response: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to deserialize UDP response: {0}")]
    InvalidResponse(#[from] rancor::Error),
}
