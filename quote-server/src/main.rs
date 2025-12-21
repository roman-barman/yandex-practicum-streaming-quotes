#![deny(unreachable_pub)]

use crate::app::{StockQuote, StockQuotesGenerator, handle_connection, quotes_generator};
use crate::tracing::initialize_tracing_subscribe;
use ::tracing::{error, info, warn};
use clap::Parser;
use crossbeam_channel::{Receiver, unbounded};
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

mod app;
mod args;
mod tracing;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::Args::parse();
    initialize_tracing_subscribe("info".into());

    let port = args.port.unwrap_or(5152);
    let address = args
        .address
        .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

    let mut threads = Vec::new();
    let is_server_working = Arc::new(AtomicBool::new(true));
    set_ctrlc_handler(Arc::clone(&is_server_working));

    let (rx, generator_thread) = crate_quotes_generator(&args, Arc::clone(&is_server_working))?;
    threads.push(generator_thread);

    let tcp_listener = TcpListener::bind((address, port))?;
    tcp_listener.set_nonblocking(true)?;

    let udp_socket = Arc::new(UdpSocket::bind((address, port))?);
    udp_socket.set_nonblocking(true)?;

    info!("Listening on {}:{}", address, port);

    for stream in tcp_listener.incoming() {
        if !is_server_working.load(Ordering::SeqCst) {
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
        let is_server_working_copy = Arc::clone(&is_server_working);
        let udp_socket_copy = Arc::clone(&udp_socket);
        let rx_copy = rx.clone();

        let handler_thread = thread::spawn(|| {
            let socket_addr = match stream.peer_addr() {
                Ok(addr) => addr,
                Err(e) => {
                    warn!("Failed to get peer address: {}", e);
                    return;
                }
            };
            info!("Accepted connection from {}", socket_addr);

            match handle_connection(stream, is_server_working_copy, udp_socket_copy, rx_copy) {
                Ok(()) => info!("Connection closed for {}", socket_addr),
                Err(e) => {
                    warn!("{}", e);
                }
            }
        });
        threads.push(handler_thread);
    }

    is_server_working.store(false, Ordering::SeqCst);

    for thread in threads {
        thread.join().expect("Failed to join thread");
    }

    info!("Server stopped");

    Ok(())
}

fn crate_quotes_generator(
    args: &args::Args,
    is_server_working: Arc<AtomicBool>,
) -> Result<(Receiver<Vec<StockQuote>>, JoinHandle<()>), std::io::Error> {
    let file = std::fs::File::open(&args.tickers_file)?;
    let generator = StockQuotesGenerator::read_from(file)?;

    let (tx, rx) = unbounded::<Vec<StockQuote>>();
    let thread = thread::spawn(move || quotes_generator(generator, tx, is_server_working));

    Ok((rx, thread))
}

fn set_ctrlc_handler(is_server_working: Arc<AtomicBool>) {
    ctrlc::set_handler(move || {
        is_server_working.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
}
