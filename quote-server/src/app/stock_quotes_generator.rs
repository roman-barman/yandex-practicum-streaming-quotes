use quote_streaming::StockQuote;
use std::io::{BufRead, BufReader, Read};

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
