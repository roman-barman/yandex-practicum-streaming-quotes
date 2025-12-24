mod handler;
mod listen;
mod monitoring;
mod quotes_generator;
mod server_cancellation_token;

use crate::app::listen::run_listening;
use crate::app::monitoring::run_monitoring;
use crate::app::quotes_generator::run_quotes_generator;
use crate::app::server_cancellation_token::ServerCancellationToken;
use crossbeam_channel::Receiver;
use std::net::{IpAddr, TcpListener, UdpSocket};
use std::sync::Arc;
use std::thread::JoinHandle;
use tracing::{info, trace};

pub(super) struct App {
    threads: Vec<JoinHandle<()>>,
    cancellation_token: Arc<ServerCancellationToken>,
}

impl App {
    pub(super) fn new() -> Self {
        Self {
            threads: Vec::new(),
            cancellation_token: Arc::new(ServerCancellationToken::default()),
        }
    }

    pub(super) fn run(
        mut self,
        address: IpAddr,
        port: u16,
        tickers: Vec<String>,
    ) -> Result<(), std::io::Error> {
        let tcp_listener = TcpListener::bind((address, port))?;
        tcp_listener.set_nonblocking(true)?;

        let udp_socket = Arc::new(UdpSocket::bind((address, port))?);
        udp_socket.set_nonblocking(true)?;

        set_ctrlc_handler(Arc::clone(&self.cancellation_token));

        let monitoring_rx = self.run_monitoring(Arc::clone(&udp_socket));
        let quote_rx = self.run_quotes_generator(tickers);
        let thread_rx = self.run_listening(
            tcp_listener,
            Arc::clone(&udp_socket),
            quote_rx,
            monitoring_rx,
        );

        self.check_treads(thread_rx);

        info!("Server stopped");

        Ok(())
    }

    fn check_treads(mut self, thread_rx: Receiver<JoinHandle<()>>) {
        loop {
            if self.cancellation_token.is_cancelled() {
                break;
            }

            match thread_rx.try_recv() {
                Ok(thread) => self.threads.push(thread),
                Err(crossbeam_channel::TryRecvError::Empty) => {
                    std::thread::sleep(std::time::Duration::from_millis(100))
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    self.cancellation_token.cancel();
                    break;
                }
            }

            let mut threads_to_delete = vec![];

            for (i, thread) in self.threads.iter().enumerate() {
                if thread.is_finished() {
                    threads_to_delete.push(i);
                }
            }

            for i in threads_to_delete.into_iter().rev() {
                let thread = self.threads.swap_remove(i);
                trace!("Joining thread");
                thread.join().expect("Failed to join thread");
            }
        }

        for (i, thread) in self.threads.into_iter().enumerate() {
            trace!("Waiting for thread {} to finish", i);
            thread.join().expect("Failed to join thread");
        }
    }

    fn run_monitoring(&mut self, udp_socket: Arc<UdpSocket>) -> Receiver<(IpAddr, u16)> {
        let (monitoring_rx, monitoring_thread) =
            run_monitoring(Arc::clone(&self.cancellation_token), udp_socket);
        self.threads.push(monitoring_thread);
        monitoring_rx
    }

    fn run_quotes_generator(
        &mut self,
        tickers: Vec<String>,
    ) -> Receiver<quote_streaming::StockQuote> {
        let (quote_rx, generator_thread) =
            run_quotes_generator(tickers, Arc::clone(&self.cancellation_token));
        self.threads.push(generator_thread);
        quote_rx
    }

    fn run_listening(
        &mut self,
        tcp_listener: TcpListener,
        udp_socket: Arc<UdpSocket>,
        quote_rx: Receiver<quote_streaming::StockQuote>,
        monitoring_rx: Receiver<(IpAddr, u16)>,
    ) -> Receiver<JoinHandle<()>> {
        let (thread_rx, listen_thread) = run_listening(
            tcp_listener,
            Arc::clone(&self.cancellation_token),
            udp_socket,
            quote_rx,
            monitoring_rx,
        );
        self.threads.push(listen_thread);
        thread_rx
    }
}

fn set_ctrlc_handler(cancellation_token: Arc<ServerCancellationToken>) {
    ctrlc::set_handler(move || {
        cancellation_token.cancel();
    })
    .expect("Error setting Ctrl-C handler");
}
