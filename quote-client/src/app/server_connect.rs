use quote_streaming::{Request, Response};
use std::io::{Read, Write};
use std::net::{IpAddr, TcpStream};
use std::time::Duration;

const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) fn connect(
    tickers: Vec<String>,
    server_address: IpAddr,
    server_port: u16,
    client_address: IpAddr,
    client_port: u16,
) -> Result<(), ServerConnectError> {
    let request = Request::StreamTickers {
        ticker: tickers,
        address: client_address,
        port: client_port,
    };
    let bytes: Vec<u8> = request
        .try_into()
        .map_err(ServerConnectError::Serialization)?;

    let mut stream = TcpStream::connect(format!("{}:{}", server_address, server_port))?;
    stream.set_read_timeout(Some(DEFAULT_READ_TIMEOUT))?;
    stream.write_all(bytes.as_slice())?;

    let mut buffer = [0; 1024];
    let len = stream.read(&mut buffer)?;

    let response = Response::try_from(&buffer[..len]);
    match response {
        Ok(Response::Ok) => Ok(()),
        Ok(Response::Error(err)) => Err(ServerConnectError::SubscriptionFailed(err)),
        Err(e) => Err(ServerConnectError::InvalidResponse(e)),
        _ => Err(ServerConnectError::UnexpectedResponse),
    }
}
#[derive(Debug, thiserror::Error)]
pub(crate) enum ServerConnectError {
    #[error("Failed to serialize command")]
    Serialization(rancor::Error),
    #[error("Failed to connect to server")]
    Connection(#[from] std::io::Error),
    #[error("Failed to subscribe to tickers: {0}")]
    SubscriptionFailed(String),
    #[error("Server returned invalid response")]
    InvalidResponse(rancor::Error),
    #[error("Server returned unexpected response")]
    UnexpectedResponse,
}
