use crate::app::server_cancellation_token::ServerCancellationToken;
use crossbeam_channel::{Receiver, Sender, unbounded};
use quote_streaming::StockQuote;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use tracing::{error, instrument, trace};

const DEFAULT_GENERATION_INTERVAL: Duration = Duration::from_secs(3);

#[instrument(name = "Run quotes generator", skip_all)]
pub(crate) fn run_quotes_generator(
    tickers: Vec<String>,
    cancellation_token: Arc<ServerCancellationToken>,
) -> (Receiver<StockQuote>, JoinHandle<()>) {
    let (tx, rx) = unbounded::<StockQuote>();
    let thread = thread::spawn(move || {
        quotes_generator(tx, cancellation_token, tickers, DEFAULT_GENERATION_INTERVAL)
    });

    (rx, thread)
}

#[instrument(name = "Generate quotes", skip_all)]
fn quotes_generator(
    tx: Sender<StockQuote>,
    cancellation_token: Arc<ServerCancellationToken>,
    tickers: Vec<String>,
    interval: Duration,
) {
    while !cancellation_token.is_cancelled() {
        for ticker in tickers.iter() {
            if cancellation_token.is_cancelled() {
                return;
            }
            let quote = StockQuote::generate(ticker);
            match tx.send(quote) {
                Ok(_) => trace!("Quote {} was generated and sent", ticker),
                Err(_) => {
                    error!("Failed to send quotes");
                    cancellation_token.cancel();
                    return;
                }
            }
        }
        thread::sleep(interval);
    }
}
