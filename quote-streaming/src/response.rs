use crate::StockQuote;
use crate::bytes::{from_bytes, to_bytes};
use rkyv::{Archive, Deserialize, Serialize, rancor};

/// Represents a response from the quote streaming server to a client.
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Response {
    /// A single stock quote update.
    Quote(StockQuote),
    /// A response to a ping request.
    Pong,
    /// An error message indicating something went wrong.
    Error(String),
    /// A generic successful response.
    Ok,
}

impl TryFrom<Response> for Vec<u8> {
    type Error = rancor::Error;
    fn try_from(value: Response) -> Result<Self, Self::Error> {
        to_bytes(&value)
    }
}

impl TryFrom<&[u8]> for Response {
    type Error = rancor::Error;
    fn try_from(value: &[u8]) -> Result<Self, rancor::Error> {
        from_bytes(value)
    }
}
