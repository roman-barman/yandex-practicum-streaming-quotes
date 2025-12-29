#![deny(unreachable_pub)]

mod app;
mod args;

use crate::app::App;
use crate::args::{Args, TickersSource};
use clap::Parser;
use std::io::BufRead;
use std::net::{IpAddr, Ipv4Addr};

const DEFAULT_CLIENT_PORT: u16 = 5153;
const DEFAULT_CLIENT_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

fn main() {
    let args = Args::parse();
    let tickers = get_tickers(args.tickers);
    let client_port = args.udp_port.unwrap_or(DEFAULT_CLIENT_PORT);
    let server_address = args.server_address;
    let server_port = args.server_port;

    let app = App::new(
        server_address,
        server_port,
        tickers,
        DEFAULT_CLIENT_ADDRESS,
        client_port,
    );
    if let Err(e) = app.run() {
        eprintln!("{}", e);
    }
}

fn get_tickers(source: TickersSource) -> Vec<String> {
    match source {
        TickersSource::File { tickers_file } => {
            let tickers_file =
                std::fs::File::open(tickers_file).expect("Failed to open tickers file");
            let reader = std::io::BufReader::new(tickers_file);
            reader
                .lines()
                .map(|line| line.expect("Cannot read line from tickers file"))
                .collect()
        }
        TickersSource::Args { tickers } => tickers,
    }
}
