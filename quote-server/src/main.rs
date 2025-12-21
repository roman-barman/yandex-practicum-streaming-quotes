#![deny(unreachable_pub)]

use crate::app::{StockQuotesGenerator, handle_connection};
use crate::tracing::initialize_tracing_subscribe;
use ::tracing::{info, warn};
use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::thread;

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

        thread::spawn(|| {
            let socket_addr = match stream.peer_addr() {
                Ok(addr) => addr,
                Err(e) => {
                    warn!("Failed to get peer address: {}", e);
                    return;
                }
            };
            info!("Accepted connection from {}", socket_addr);

            match handle_connection(stream) {
                Ok(()) => info!("Connection closed for {}", socket_addr),
                Err(e) => {
                    warn!("{}", e);
                }
            }
        });
    }

    Ok(())
}
