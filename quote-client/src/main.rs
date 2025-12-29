#![deny(unreachable_pub)]

mod args;
mod cancellation_token;

use crate::args::{Args, TickersSource};
use crate::cancellation_token::CancellationToken;
use clap::Parser;
use quote_streaming::{Request, Response};
use rancor::Error;
use std::io::{BufRead, ErrorKind, Read, Write};
use std::net::{IpAddr, Ipv4Addr, TcpStream, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const DEFAULT_CLIENT_PORT: u16 = 5153;
const DEFAULT_CLIENT_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const DEFAULT_SOCKET_TIMEOUT: Duration = Duration::from_secs(5);
const PING_INTERVAL: Duration = Duration::from_secs(2);

fn main() {
    let args = Args::parse();
    let cancellation_token = Arc::new(CancellationToken::default());
    let tickers = get_tickers(args.tickers);
    let client_port = args.udp_port.unwrap_or(DEFAULT_CLIENT_PORT);
    let server_address = args.server_address;
    let server_port = args.server_port;

    show_app_title(server_address, server_port, &tickers);

    let socket = Arc::new(
        UdpSocket::bind((DEFAULT_CLIENT_ADDRESS, client_port)).expect("Failed to bind UDP socket"),
    );
    socket
        .set_read_timeout(Some(DEFAULT_SOCKET_TIMEOUT))
        .expect("Failed to set read timeout");
    socket
        .set_write_timeout(Some(DEFAULT_SOCKET_TIMEOUT))
        .expect("Failed to set write timeout");

    set_ctrlc_handler(Arc::clone(&cancellation_token));

    send_request(
        tickers,
        server_address,
        server_port,
        DEFAULT_CLIENT_ADDRESS,
        client_port,
    );

    let ping_thread = send_ping(
        Arc::clone(&socket),
        Arc::clone(&cancellation_token),
        server_address,
        server_port,
    );

    read_responses(cancellation_token, socket);

    ping_thread.join().expect("Failed to join ping thread");
}

fn show_app_title(server_address: IpAddr, server_port: u16, tickers: &[String]) {
    println!("Client started");
    println!("======================");
    println!("Server address  {}:{}", server_address, server_port);
    println!("Tickers         {}", tickers.join(", "));
    println!("======================");
}

fn read_responses(cancellation_token: Arc<CancellationToken>, socket: Arc<UdpSocket>) {
    let mut buffer = [0; 1024];
    while !cancellation_token.is_cancelled() {
        let len = match socket.recv(&mut buffer) {
            Ok(read_bytes) => read_bytes,
            Err(e) => {
                if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock {
                    eprintln!("Server disconnected");
                } else {
                    eprintln!("Failed to receive response: {}", e);
                }

                cancellation_token.cancel();
                break;
            }
        };
        match deserialize_response(&buffer[..len]) {
            Ok(Response::Quote(quote)) => println!("{}", quote),
            Ok(Response::Pong) | Ok(Response::Ok) => {}
            Ok(Response::Error(err)) => println!("Server send error: {}", err),
            Err(e) => {
                eprintln!("Failed to deserialize response: {}", e);
                cancellation_token.cancel();
                break;
            }
        }
    }
}

fn deserialize_response(bytes: &[u8]) -> Result<Response, Error> {
    rkyv::from_bytes::<Response, rancor::Error>(bytes)
}

fn send_request(
    tickers: Vec<String>,
    server_address: IpAddr,
    server_port: u16,
    client_address: IpAddr,
    client_port: u16,
) {
    let command = Request::StreamTickers {
        ticker: tickers,
        address: client_address,
        port: client_port,
    };
    let bytes = rkyv::to_bytes::<Error>(&command).expect("Failed to serialize command");

    let mut stream = TcpStream::connect(format!("{}:{}", server_address, server_port))
        .expect("Failed to connect to server");
    stream
        .write_all(bytes.as_slice())
        .expect("Failed to send request to server");

    let mut buffer = [0; 1024];
    let len = stream
        .read(&mut buffer)
        .expect("Failed to read response from server");
    let response = deserialize_response(&buffer[..len]);
    match response {
        Ok(Response::Ok) => println!("Successfully subscribed to tickers"),
        Ok(Response::Error(err)) => println!("Failed to subscribe to tickers: {}", err),
        Err(e) => println!("Failed to deserialize response: {}", e),
        _ => {}
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

fn send_ping(
    socket: Arc<UdpSocket>,
    cancellation_token: Arc<CancellationToken>,
    sender_address: IpAddr,
    server_port: u16,
) -> thread::JoinHandle<()> {
    let ping = rkyv::to_bytes::<Error>(&Request::Ping).expect("Failed to serialize ping request");
    thread::spawn(move || {
        while !cancellation_token.is_cancelled() {
            if let Err(e) = socket.send_to(&ping, (sender_address, server_port)) {
                if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock {
                    eprintln!("Server disconnected");
                } else {
                    eprintln!("Failed to send ping: {}", e);
                }
                cancellation_token.cancel();
                break;
            }
            thread::sleep(PING_INTERVAL);
        }
    })
}

fn set_ctrlc_handler(cancellation_token: Arc<CancellationToken>) {
    ctrlc::set_handler(move || {
        cancellation_token.cancel();
    })
    .expect("Error setting Ctrl-C handler");
}
