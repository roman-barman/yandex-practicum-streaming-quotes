use crate::app::StockQuote;
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
