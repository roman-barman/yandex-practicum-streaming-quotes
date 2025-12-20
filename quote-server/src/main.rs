#![deny(unreachable_pub)]

use crate::stock_quotes_generator::StockQuotesGenerator;
use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, TcpListener};

mod args;
mod stock_quote;
mod stock_quotes_generator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::Args::parse();
    let port = args.port.unwrap_or(8080);
    let address = args
        .address
        .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

    let listener = TcpListener::bind((address, port))?;
    let file = std::fs::File::open(&args.tickers_file)?;
    let generator = StockQuotesGenerator::read_from(file)?;

    for stream in listener.incoming() {
        println!("Got a connection!");
    }

    Ok(())
}
