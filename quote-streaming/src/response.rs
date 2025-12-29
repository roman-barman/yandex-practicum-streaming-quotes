use crate::StockQuote;
use crate::bytes::{from_bytes, to_bytes};
use rkyv::{Archive, Deserialize, Serialize, rancor};

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Response {
    Quote(StockQuote),
    Pong,
    Error(String),
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
