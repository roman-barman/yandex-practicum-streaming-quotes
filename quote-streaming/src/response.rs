use crate::StockQuote;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Response {
    Quote(StockQuote),
    Pong,
    Error(String),
    Ok,
}
