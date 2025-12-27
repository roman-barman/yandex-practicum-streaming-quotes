use rkyv::{Archive, Deserialize, Serialize};
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
