use crate::bytes::{from_bytes, to_bytes};
use rkyv::{Archive, Deserialize, Serialize, rancor};
use std::net::IpAddr;

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Request {
    StreamTickers {
        ticker: Vec<String>,
        address: IpAddr,
        port: u16,
    },
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
