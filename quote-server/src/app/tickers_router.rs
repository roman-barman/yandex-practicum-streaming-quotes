use crate::app::client_address::ClientAddress;
use crossbeam_channel::Sender;
use quote_streaming::StockQuote;
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use tracing::instrument;

pub(crate) struct TickersRouter {
    tickers_router: RwLock<HashMap<String, HashMap<ClientAddress, Sender<StockQuote>>>>,
    client_tickers: RwLock<HashMap<ClientAddress, Vec<String>>>,
}

impl TickersRouter {
    pub(crate) fn new(tickers: Vec<String>) -> Self {
        let tickers_router = tickers
            .into_iter()
            .map(|ticker| (ticker, HashMap::new()))
            .collect();
        Self {
            tickers_router: RwLock::new(tickers_router),
            client_tickers: RwLock::new(HashMap::new()),
        }
    }

    #[instrument(name = "Add quote route", skip(self, tx), fields(address, tickers = ?ticker))]
    pub(crate) fn add_routes(
        &self,
        ticker: &[&str],
        tx: Sender<StockQuote>,
        client_address: ClientAddress,
    ) -> Result<(), TickersRouterError> {
        let mut route_lock = self
            .tickers_router
            .write()
            .map_err(|e| TickersRouterError::RwLockPoisoned(e.to_string()))?;
        let mut client_lock = self
            .client_tickers
            .write()
            .map_err(|e| TickersRouterError::RwLockPoisoned(e.to_string()))?;

        let not_found = ticker
            .iter()
            .find(|ticker| route_lock.get(**ticker).is_none())
            .map(|ticker| ticker.to_string());

        if let Some(not_found) = not_found {
            return Err(TickersRouterError::TickerNotFound(not_found));
        }

        for ticker in ticker {
            if let Some(clients) = route_lock.get_mut(*ticker) {
                clients.insert(client_address.clone(), tx.clone());
                client_lock
                    .entry(client_address.clone())
                    .or_default()
                    .push(ticker.to_string());
            } else {
                return Err(TickersRouterError::Unexpect);
            }
        }
        Ok(())
    }

    #[instrument(name = "Send quote", skip(self), fields(ticker = quote.ticker()))]
    pub(crate) fn send_quote(&self, quote: StockQuote) -> Result<(), TickersRouterError> {
        let mut delete_client = HashSet::new();

        {
            let lock = self
                .tickers_router
                .read()
                .map_err(|e| TickersRouterError::RwLockPoisoned(e.to_string()))?;

            if let Some(clients) = lock.get(quote.ticker()) {
                for (address, tx) in clients {
                    if tx.send(quote.clone()).is_err() {
                        delete_client.insert(address.clone());
                    }
                }
            }
        }

        self.delete_clients(delete_client)?;

        Ok(())
    }

    #[instrument(name = "Get tickers with subscribers", skip(self))]
    pub(crate) fn get_tickers_with_subscribers(&self) -> Result<Vec<String>, TickersRouterError> {
        let lock = self
            .tickers_router
            .read()
            .map_err(|e| TickersRouterError::RwLockPoisoned(e.to_string()))?;

        let result = lock
            .iter()
            .filter(|(_, clients)| !clients.is_empty())
            .map(|(ticker, _)| ticker.clone())
            .collect();

        Ok(result)
    }

    #[instrument(name = "Delete clients from quote route", skip(self), fields(clients = ?clients))]
    pub(crate) fn delete_clients(
        &self,
        clients: HashSet<ClientAddress>,
    ) -> Result<(), TickersRouterError> {
        if clients.is_empty() {
            return Ok(());
        }

        let mut route_lock = self
            .tickers_router
            .write()
            .map_err(|e| TickersRouterError::RwLockPoisoned(e.to_string()))?;
        let mut client_lock = self
            .client_tickers
            .write()
            .map_err(|e| TickersRouterError::RwLockPoisoned(e.to_string()))?;

        for client_address in clients.into_iter() {
            if let Some(tickers) = client_lock.remove(&client_address) {
                for ticker in tickers {
                    route_lock.entry(ticker).and_modify(|clients| {
                        clients.remove(&client_address);
                        println!(
                            "Removed client from ticker route: client={}",
                            client_address
                        );
                    });
                }
            }
        }
        Ok(())
    }
}
#[derive(Debug, thiserror::Error)]
pub(crate) enum TickersRouterError {
    #[error("Failed to get lock: {0}")]
    RwLockPoisoned(String),
    #[error("Ticker {0} not found")]
    TickerNotFound(String),
    #[error("Unexpected error")]
    Unexpect,
}
