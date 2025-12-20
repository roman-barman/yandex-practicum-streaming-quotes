#![deny(unreachable_pub)]

use crate::app::StockQuotesGenerator;
use crate::tracing::initialize_tracing_subscribe;
use ::tracing::{info, instrument, warn};
use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, TcpListener, TcpStream};

mod app;
mod args;
mod tracing;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::Args::parse();
    initialize_tracing_subscribe("trace".into());

    let port = args.port.unwrap_or(5152);
    let address = args
        .address
        .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

    let listener = TcpListener::bind((address, port))?;
    let file = std::fs::File::open(&args.tickers_file)?;
    let generator = StockQuotesGenerator::read_from(file)?;

    info!("Listening on {}:{}", address, port);
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(e) => {
                warn!("Failed to accept connection: {}", e);
                continue;
            }
        };
    }

    Ok(())
}

#[instrument(name = "Handle connection", skip(stream))]
fn handle_connection(stream: &mut TcpStream) {}
