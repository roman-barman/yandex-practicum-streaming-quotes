#![deny(unreachable_pub)]

use crate::app::App;
use crate::args::Args;
use crate::tracing::initialize_tracing_subscribe;
use clap::Parser;
use std::io::{BufRead, BufReader};
use std::net::{IpAddr, Ipv4Addr};

mod app;
mod args;
mod tracing;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    initialize_tracing_subscribe(args.log_level());

    let port = args.port.unwrap_or(5152);
    let address = args
        .address
        .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    let tickers = read_tickers(&args)?;

    let app = App::new(address, port, tickers);
    app.run();

    Ok(())
}

fn read_tickers(args: &Args) -> Result<Vec<String>, std::io::Error> {
    let file = std::fs::File::open(&args.tickers_file)?;
    let buffer = BufReader::new(file);
    let mut tickers = Vec::new();
    for line in buffer.lines() {
        tickers.push(line?.trim().to_string());
    }
    Ok(tickers)
}
