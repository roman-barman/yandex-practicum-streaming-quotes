use crate::app::{StockQuote, StockQuotesGenerator};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;
use tracing::{error, info, instrument};

#[instrument(name = "Generate quotes", skip_all)]
pub(crate) fn quotes_generator(
    generator: StockQuotesGenerator,
    tx: std::sync::mpsc::Sender<Vec<StockQuote>>,
    flag: Arc<AtomicBool>,
) {
    loop {
        let quotes = generator.generate();
        match tx.send(quotes) {
            Ok(_) => info!("Quotes was generated and sent"),
            Err(_) => {
                error!("Failed to send quotes");
                flag.store(false, std::sync::atomic::Ordering::Relaxed);
                return;
            }
        }
        thread::sleep(Duration::from_secs(3));
    }
}
