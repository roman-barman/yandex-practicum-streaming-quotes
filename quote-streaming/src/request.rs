use crate::bytes::{from_bytes, to_bytes};
use rkyv::{Archive, Deserialize, Serialize, rancor};
use std::net::IpAddr;

/// Represents a request from a client to the quote streaming server.
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Request {
    /// A request to start streaming quotes for a list of tickers.
    StreamTickers {
        /// The list of ticker symbols to stream.
        ticker: Vec<String>,
        /// The IP address of the client to send the quotes to.
        address: IpAddr,
        /// The port number of the client to send the quotes to.
        port: u16,
    },
    /// A simple ping request to check server availability.
    Ping,
}

impl TryFrom<Request> for Vec<u8> {
    type Error = rancor::Error;
    fn try_from(value: Request) -> Result<Self, Self::Error> {
        to_bytes(&value)
    }
}

impl TryFrom<&[u8]> for Request {
    type Error = rancor::Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        from_bytes(value)
    }
}
