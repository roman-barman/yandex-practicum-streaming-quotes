use crate::app::{StockQuote, StockQuotesGenerator};
use crossbeam_channel::Sender;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;
use tracing::{error, instrument, trace};

#[instrument(name = "Generate quotes", skip_all)]
pub(crate) fn quotes_generator(
    generator: StockQuotesGenerator,
    tx: Sender<Vec<StockQuote>>,
    is_server_working: Arc<AtomicBool>,
) {
    loop {
        if !is_server_working.load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }

        let quotes = generator.generate();
        match tx.send(quotes) {
            Ok(_) => trace!("Quotes was generated and sent"),
            Err(_) => {
                error!("Failed to send quotes");
                is_server_working.store(false, std::sync::atomic::Ordering::SeqCst);
                return;
            }
        }
        thread::sleep(Duration::from_secs(3));
    }
}
