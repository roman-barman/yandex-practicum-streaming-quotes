mod handler;
mod monitoring;
mod quotes_generator;
mod server_cancellation_token;

use crate::app::handler::accept_connection;
use crate::app::monitoring::run_monitoring;
use crate::app::quotes_generator::run_quotes_generator;
use crate::app::server_cancellation_token::ServerCancellationToken;
use std::io::ErrorKind;
use std::net::{IpAddr, TcpListener, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use tracing::{error, info};

pub(super) struct App {
    address: IpAddr,
    port: u16,
    threads: Vec<JoinHandle<()>>,
    cancellation_token: Arc<ServerCancellationToken>,
    tickers: Vec<String>,
}

impl App {
    pub(super) fn new(address: IpAddr, port: u16, tickers: Vec<String>) -> Self {
        Self {
            address,
            port,
            threads: Vec::new(),
            cancellation_token: Arc::new(ServerCancellationToken::default()),
            tickers,
        }
    }

    pub(super) fn run(mut self) -> Result<(), std::io::Error> {
        let tcp_listener = TcpListener::bind((self.address, self.port))?;
        tcp_listener.set_nonblocking(true)?;

        let udp_socket = Arc::new(UdpSocket::bind((self.address, self.port))?);
        udp_socket.set_nonblocking(true)?;

        info!("Listening on {}:{}", self.address, self.port);

        set_ctrlc_handler(Arc::clone(&self.cancellation_token));

        let (quote_rx, generator_thread) =
            run_quotes_generator(self.tickers, Arc::clone(&self.cancellation_token));
        self.threads.push(generator_thread);

        let (monitoring_rx, monitoring_thread) = run_monitoring(
            Arc::clone(&self.cancellation_token),
            Arc::clone(&udp_socket),
        );
        self.threads.push(monitoring_thread);

        for stream in tcp_listener.incoming() {
            if self.cancellation_token.is_cancelled() {
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

            self.threads.push(accept_connection(
                stream,
                Arc::clone(&self.cancellation_token),
                Arc::clone(&udp_socket),
                quote_rx.clone(),
                monitoring_rx.clone(),
            ));
        }

        self.cancellation_token.cancel();

        for thread in self.threads {
            thread.join().expect("Failed to join thread");
        }

        info!("Server stopped");

        Ok(())
    }
}

fn set_ctrlc_handler(cancellation_token: Arc<ServerCancellationToken>) {
    ctrlc::set_handler(move || {
        cancellation_token.cancel();
    })
    .expect("Error setting Ctrl-C handler");
}
