use crate::app::StockQuotesGenerator;
use crate::app::server_cancellation_token::ServerCancellationToken;
use crossbeam_channel::Sender;
use quote_streaming::StockQuote;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::{error, instrument, trace};

#[instrument(name = "Generate quotes", skip_all)]
pub(crate) fn quotes_generator(
    generator: StockQuotesGenerator,
    tx: Sender<Vec<StockQuote>>,
    cancellation_token: Arc<ServerCancellationToken>,
) {
    loop {
        if cancellation_token.is_cancelled() {
            return;
        }

        let quotes = generator.generate();
        match tx.send(quotes) {
            Ok(_) => trace!("Quotes was generated and sent"),
            Err(_) => {
                error!("Failed to send quotes");
                cancellation_token.cancel();
                return;
            }
        }
        thread::sleep(Duration::from_secs(3));
    }
}
