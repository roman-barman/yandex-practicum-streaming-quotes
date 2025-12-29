//! Quote streaming library for handling stock quotes, requests, and responses.
//!
//! This crate provides the common data structures and serialization logic
//! used by both the quote server and quote client.

#![deny(unreachable_pub)]
#![warn(missing_docs)]
mod bytes;
mod request;
mod response;
mod stock_quote;

/// Request types for the quote streaming service.
pub use request::Request;
/// Response types from the quote streaming service.
pub use response::Response;
/// Stock quote data structure.
pub use stock_quote::StockQuote;
