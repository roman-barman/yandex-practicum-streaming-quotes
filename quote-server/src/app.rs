mod client_address;
mod handler;
mod listen;
mod monitoring;
mod monitoring_router;
mod quotes_generator;
mod server_cancellation_token;
mod tickers_router;

use crate::app::client_address::ClientAddress;
use crate::app::listen::{ListenContext, run_listening};
use crate::app::monitoring::run_monitoring;
use crate::app::monitoring_router::MonitoringRouter;
use crate::app::quotes_generator::run_quotes_generator;
use crate::app::server_cancellation_token::ServerCancellationToken;
use crate::app::tickers_router::TickersRouter;
use crossbeam_channel::Receiver;
use std::collections::HashSet;
use std::net::{IpAddr, TcpListener, UdpSocket};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use tracing::{info, trace};

const STOP_DURATION: Duration = Duration::from_millis(100);

pub(super) struct App {
    service_threads: Vec<JoinHandle<()>>,
    client_threads: Vec<JoinHandle<Option<ClientAddress>>>,
    cancellation_token: Arc<ServerCancellationToken>,
}

impl App {
    pub(super) fn new() -> Self {
        Self {
            service_threads: Vec::new(),
            client_threads: Vec::new(),
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
        udp_socket.set_read_timeout(Some(Duration::from_millis(500)))?;

        let tickers_router = Arc::new(TickersRouter::new(tickers));
        let monitoring_router = Arc::new(MonitoringRouter::default());

        set_ctrlc_handler(Arc::clone(&self.cancellation_token));

        self.run_monitoring(Arc::clone(&udp_socket), Arc::clone(&monitoring_router));
        self.run_quotes_generator(Arc::clone(&tickers_router));
        let thread_rx = self.run_listening(
            tcp_listener,
            Arc::clone(&udp_socket),
            Arc::clone(&tickers_router),
            Arc::clone(&monitoring_router),
        );

        self.check_treads(thread_rx, monitoring_router, tickers_router);

        info!("Server stopped");

        Ok(())
    }

    fn check_treads(
        mut self,
        client_thread_rx: Receiver<JoinHandle<Option<ClientAddress>>>,
        monitoring_router: Arc<MonitoringRouter>,
        tickers_router: Arc<TickersRouter>,
    ) {
        while !self.cancellation_token.is_cancelled() {
            match client_thread_rx.try_recv() {
                Ok(thread) => self.client_threads.push(thread),
                Err(crossbeam_channel::TryRecvError::Empty) => std::thread::sleep(STOP_DURATION),
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    self.cancellation_token.cancel();
                    break;
                }
            }
            self.check_clients_threads(&monitoring_router, &tickers_router);
        }

        self.join_threads();
    }

    fn check_clients_threads(
        &mut self,
        monitoring_router: &Arc<MonitoringRouter>,
        tickers_router: &Arc<TickersRouter>,
    ) {
        let mut threads_to_delete = vec![];

        for (i, thread) in self.client_threads.iter().enumerate() {
            if thread.is_finished() {
                threads_to_delete.push(i);
            }
        }

        for i in threads_to_delete.into_iter().rev() {
            let thread = self.client_threads.swap_remove(i);
            trace!("Joining thread");
            let client_address = thread.join().expect("Failed to join thread");
            if let Some(address) = client_address {
                monitoring_router
                    .delete(&address)
                    .expect("Failed to delete client from monitoring router");
                tickers_router
                    .delete_clients(HashSet::from([address]))
                    .expect("Failed to delete clients from tickers router");
            }
        }
    }

    fn join_threads(self) {
        for (i, thread) in self.client_threads.into_iter().enumerate() {
            trace!("Waiting for client thread {} to finish", i);
            thread.join().expect("Failed to join thread");
        }

        for (i, thread) in self.service_threads.into_iter().enumerate() {
            trace!("Waiting for service thread {} to finish", i);
            thread.join().expect("Failed to join thread");
        }
    }

    fn run_monitoring(
        &mut self,
        udp_socket: Arc<UdpSocket>,
        monitoring_router: Arc<MonitoringRouter>,
    ) {
        let monitoring_thread = run_monitoring(
            Arc::clone(&self.cancellation_token),
            udp_socket,
            monitoring_router,
        );
        self.service_threads.push(monitoring_thread);
    }

    fn run_quotes_generator(&mut self, tickers_router: Arc<TickersRouter>) {
        let generator_thread =
            run_quotes_generator(tickers_router, Arc::clone(&self.cancellation_token));
        self.service_threads.push(generator_thread);
    }

    fn run_listening(
        &mut self,
        tcp_listener: TcpListener,
        udp_socket: Arc<UdpSocket>,
        tickers_router: Arc<TickersRouter>,
        monitoring_router: Arc<MonitoringRouter>,
    ) -> Receiver<JoinHandle<Option<ClientAddress>>> {
        let context = ListenContext::new(
            tcp_listener,
            Arc::clone(&self.cancellation_token),
            udp_socket,
            tickers_router,
            monitoring_router,
        );

        let (thread_rx, listen_thread) = run_listening(context);
        self.service_threads.push(listen_thread);
        thread_rx
    }
}

fn set_ctrlc_handler(cancellation_token: Arc<ServerCancellationToken>) {
    ctrlc::set_handler(move || {
        cancellation_token.cancel();
    })
    .expect("Error setting Ctrl-C handler");
}
