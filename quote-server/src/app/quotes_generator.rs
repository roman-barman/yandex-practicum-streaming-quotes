use crate::app::server_cancellation_token::ServerCancellationToken;
use crate::app::tickers_router::TickersRouter;
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
    tickers_router: Arc<TickersRouter>,
    cancellation_token: Arc<ServerCancellationToken>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        quotes_generator(
            cancellation_token,
            tickers_router,
            DEFAULT_GENERATION_INTERVAL,
        )
    })
}

#[instrument(name = "Generate quotes", skip_all)]
fn quotes_generator(
    cancellation_token: Arc<ServerCancellationToken>,
    tickers_router: Arc<TickersRouter>,
    interval: Duration,
) {
    while !cancellation_token.is_cancelled() {
        match tickers_router.get_tickers_with_subscribers() {
            Ok(tickers) => {
                for ticker in tickers {
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
            }
            Err(e) => warn!("Failed to get tickers with subscribers: {e}"),
        }
        thread::sleep(interval);
    }
}
