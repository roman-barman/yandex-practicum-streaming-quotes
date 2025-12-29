mod cancellation_token;
mod error;
mod ping;
mod read_udp_response;
mod server_connect;

use crate::app::cancellation_token::CancellationToken;
use crate::app::read_udp_response::read_udp_response;
use crate::app::server_connect::connect;
use std::net::{IpAddr, UdpSocket};
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_SOCKET_TIMEOUT: Duration = Duration::from_secs(5);

pub(super) struct App {
    cancellation_token: Arc<CancellationToken>,
    server_address: IpAddr,
    server_port: u16,
    tickers: Vec<String>,
    client_address: IpAddr,
    client_port: u16,
}

impl App {
    pub(super) fn new(
        server_address: IpAddr,
        server_port: u16,
        tickers: Vec<String>,
        client_address: IpAddr,
        client_port: u16,
    ) -> Self {
        Self {
            cancellation_token: Arc::new(CancellationToken::default()),
            server_address,
            server_port,
            tickers,
            client_address,
            client_port,
        }
    }

    pub(super) fn run(self) -> Result<(), error::AppError> {
        set_ctrlc_handler(Arc::clone(&self.cancellation_token))?;
        self.show_app_title();
        let socket = self.create_udp_socket()?;
        connect(
            self.tickers,
            self.server_address,
            self.server_port,
            self.client_address,
            self.client_port,
        )?;
        let ping_thread = ping::start_ping(
            Arc::clone(&self.cancellation_token),
            Arc::clone(&socket),
            self.client_address,
            self.server_port,
        );
        read_udp_response(Arc::clone(&self.cancellation_token), socket)?;
        ping_thread
            .join()
            .map_err(|_| error::AppError::JoinThread("Ping thread".to_string()))?;
        Ok(())
    }

    fn show_app_title(&self) {
        println!("Client started");
        println!("======================");
        println!(
            "Server address  {}:{}",
            self.server_address, self.server_port
        );
        println!("Tickers         {}", self.tickers.join(", "));
        println!("======================");
    }

    fn create_udp_socket(&self) -> Result<Arc<UdpSocket>, error::AppError> {
        let socket = Arc::new(
            UdpSocket::bind((self.client_address, self.client_port))
                .map_err(error::AppError::UdpSocket)?,
        );
        socket
            .set_read_timeout(Some(DEFAULT_SOCKET_TIMEOUT))
            .map_err(error::AppError::UdpSocket)?;
        socket
            .set_write_timeout(Some(DEFAULT_SOCKET_TIMEOUT))
            .map_err(error::AppError::UdpSocket)?;
        Ok(socket)
    }
}

fn set_ctrlc_handler(cancellation_token: Arc<CancellationToken>) -> Result<(), ctrlc::Error> {
    ctrlc::set_handler(move || {
        cancellation_token.cancel();
    })
}
