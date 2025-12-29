use crate::app::quote_streaming::tickers_router::TickersRouter;
use crate::app::server_cancellation_token::ServerCancellationToken;
use quote_streaming::StockQuote;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use tracing::warn;
use tracing::{instrument, trace};

const DEFAULT_GENERATION_INTERVAL: Duration = Duration::from_secs(3);

#[instrument(name = "Run quotes generator", skip_all)]
pub(crate) fn run_quotes_generator(
    tickers: Vec<String>,
    tickers_router: Arc<TickersRouter>,
    cancellation_token: Arc<ServerCancellationToken>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        quotes_generator(
            tickers,
            cancellation_token,
            tickers_router,
            DEFAULT_GENERATION_INTERVAL,
        )
    })
}

#[instrument(name = "Generate quotes", skip_all)]
fn quotes_generator(
    tickers: Vec<String>,
    cancellation_token: Arc<ServerCancellationToken>,
    tickers_router: Arc<TickersRouter>,
    interval: Duration,
) {
    while !cancellation_token.is_cancelled() {
        for ticker in tickers.iter() {
            if cancellation_token.is_cancelled() {
                return;
            }
            let quote = StockQuote::generate(ticker.as_str());
            match tickers_router.send_quote(quote) {
                Ok(_) => trace!("Quote {} was generated and sent", ticker),
                Err(e) => {
                    warn!("Failed to send quotes: {e}");
                }
            }
        }
        thread::sleep(interval);
    }
}
