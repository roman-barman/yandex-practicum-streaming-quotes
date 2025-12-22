#![deny(unreachable_pub)]

use crate::app::{
    ServerCancellationToken, accept_connection, run_quotes_generator, start_monitoring,
};
use crate::tracing::initialize_tracing_subscribe;
use ::tracing::{error, info};
use clap::Parser;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};
use std::sync::Arc;
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
    let cancellation_token = Arc::new(ServerCancellationToken::default());
    set_ctrlc_handler(Arc::clone(&cancellation_token));

    let file = std::fs::File::open(&args.tickers_file)?;
    let (rx, generator_thread) = run_quotes_generator(file, Arc::clone(&cancellation_token))?;
    threads.push(generator_thread);

    let tcp_listener = TcpListener::bind((address, port))?;
    tcp_listener.set_nonblocking(true)?;

    let udp_socket = Arc::new(UdpSocket::bind((address, port))?);
    udp_socket.set_nonblocking(true)?;

    info!("Listening on {}:{}", address, port);

    let monitoring_thread =
        run_monitoring(Arc::clone(&udp_socket), Arc::clone(&cancellation_token));
    threads.push(monitoring_thread);

    for stream in tcp_listener.incoming() {
        if cancellation_token.is_cancelled() {
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

        threads.push(accept_connection(
            stream,
            Arc::clone(&cancellation_token),
            Arc::clone(&udp_socket),
            rx.clone(),
        ));
    }

    cancellation_token.cancel();

    for thread in threads {
        thread.join().expect("Failed to join thread");
    }

    info!("Server stopped");

    Ok(())
}

fn set_ctrlc_handler(cancellation_token: Arc<ServerCancellationToken>) {
    ctrlc::set_handler(move || {
        cancellation_token.cancel();
    })
    .expect("Error setting Ctrl-C handler");
}

fn run_monitoring(
    udp_socket: Arc<UdpSocket>,
    cancellation_token: Arc<ServerCancellationToken>,
) -> JoinHandle<()> {
    thread::spawn(move || start_monitoring(cancellation_token, udp_socket))
}
