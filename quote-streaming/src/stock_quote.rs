use chrono::{MappedLocalTime, TimeZone, Utc};
use rand::Rng;
use rkyv::{Archive, Deserialize, Serialize};
use rust_decimal::Decimal;
use std::fmt::{Display, Formatter};

/// Represents a single stock quote update.
#[derive(Debug, Clone, Eq, PartialEq, Archive, Deserialize, Serialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct StockQuote {
    /// The ticker symbol of the stock (e.g., "AAPL").
    ticker: String,
    /// The price of the stock, multiplied by 100 to avoid floating point issues.
    price: i64,
    /// The volume of shares traded in this update.
    volume: u32,
    /// The Unix timestamp when the quote was generated.
    timestamp: i64,
}

impl Display for StockQuote {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let date_time = Utc.timestamp_opt(self.timestamp, 0);
        let price = Decimal::new(self.price, 2);
        match date_time {
            MappedLocalTime::Single(date_time) => write!(
                f,
                "{}: {} ({}) {}",
                self.ticker, price, self.volume, date_time
            ),
            _ => write!(f, "{} {} {}", self.ticker, price, self.volume),
        }
    }
}

impl StockQuote {
    /// Generates a random `StockQuote` for the given ticker.
    ///
    /// The price is randomly generated between 1.00 and 1000.00.
    /// The volume is randomly generated between 10 and 1000.
    pub fn generate(ticker: &str) -> Self {
        let mut rng = rand::rng();
        Self {
            ticker: ticker.to_string(),
            price: rng.random_range(100..100000),
            volume: rng.random_range(10..1000),
            timestamp: Utc::now().timestamp(),
        }
    }

    /// Returns the ticker symbol of the stock.
    pub fn ticker(&self) -> &str {
        &self.ticker
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_quote() {
        let quote_1 = StockQuote::generate("AAPL");
        assert_eq!(quote_1.ticker, "AAPL");
        assert!(quote_1.price >= 100);
        assert!(quote_1.volume >= 10);
        assert!(quote_1.timestamp > 0);

        let quote_2 = StockQuote::generate("MSFT");
        assert_ne!(quote_1.ticker, quote_2.ticker);
    }
}
