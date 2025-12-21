#![deny(unreachable_pub)]

use crate::app::{StockQuote, StockQuotesGenerator, handle_connection, quotes_generator};
use crate::tracing::initialize_tracing_subscribe;
use ::tracing::{error, info, warn};
use clap::Parser;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

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

    let (is_generator_working, rx) = crate_quotes_generator(&args)?;

    let listener = TcpListener::bind((address, port))?;
    listener.set_nonblocking(true)?;
    info!("Listening on {}:{}", address, port);

    for stream in listener.incoming() {
        if !is_generator_working.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }

        let stream = match stream {
            Ok(stream) => stream,
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
                break;
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

fn crate_quotes_generator(
    args: &args::Args,
) -> Result<(Arc<AtomicBool>, mpsc::Receiver<Vec<StockQuote>>), std::io::Error> {
    let file = std::fs::File::open(&args.tickers_file)?;
    let generator = StockQuotesGenerator::read_from(file)?;

    let (tx, rx) = mpsc::channel::<Vec<StockQuote>>();
    let working = Arc::new(AtomicBool::new(true));
    let working_copy = Arc::clone(&working);
    thread::spawn(move || quotes_generator(generator, tx, working_copy));

    Ok((working, rx))
}
