use rkyv::{Archive, Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Commands {
    Stream {
        ticker: Vec<String>,
        address: IpAddr,
    },
}
