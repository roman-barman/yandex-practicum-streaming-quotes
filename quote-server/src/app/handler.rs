use quote_streaming::Commands;
use std::io::Read;
use tracing::{info, instrument, warn};

#[instrument(name = "Handle connection", skip_all)]
pub(crate) fn handle_connection<R: Read>(mut reader: R) -> Result<(), HandlerError> {
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let command = rkyv::from_bytes::<Commands, rancor::Error>(&buffer)?;
    info!("Received command: {:?}", command);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum HandlerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Deserialization command error: {0}")]
    Deserialization(#[from] rancor::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_handle_connection() {
        let command = Commands::Stream {
            ticker: vec!["AAPL".to_string(), "GOOGL".to_string()],
            address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 5152,
        };
        let bytes = rkyv::to_bytes::<rancor::Error>(&command).unwrap();
        let reader = Cursor::new(bytes);
        assert!(handle_connection(reader).is_ok());
    }
}
