use crate::app::server_cancellation_token::ServerCancellationToken;
use crossbeam_channel::Sender;
use quote_streaming::StockQuote;
use std::io::{BufRead, BufReader, Read};
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

pub(crate) struct StockQuotesGenerator {
    tickers: Vec<String>,
}

impl StockQuotesGenerator {
    pub(crate) fn read_from<R: Read>(reader: R) -> Result<Self, std::io::Error> {
        let buffer = BufReader::new(reader);
        let mut tickers = Vec::new();
        for line in buffer.lines() {
            tickers.push(line?.trim().to_string());
        }
        Ok(Self { tickers })
    }

    pub(super) fn generate(&self) -> Vec<StockQuote> {
        self.tickers
            .iter()
            .map(|ticker| StockQuote::generate(ticker))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_from() {
        let tickers = "AAPL\nMSFT\nGOOGL";
        let reader = Cursor::new(tickers);
        let generator = StockQuotesGenerator::read_from(reader).unwrap();
        assert_eq!(generator.tickers, vec!["AAPL", "MSFT", "GOOGL"]);
    }

    #[test]
    fn test_generate() {
        let generator = StockQuotesGenerator {
            tickers: vec!["AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string()],
        };
        let quotes = generator.generate();
        assert_eq!(quotes.len(), 3);
    }
}
