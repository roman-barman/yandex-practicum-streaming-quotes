use chrono::{MappedLocalTime, TimeZone, Utc};
use rand::Rng;
use rkyv::{Archive, Deserialize, Serialize};
use rust_decimal::Decimal;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Eq, PartialEq, Archive, Deserialize, Serialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct StockQuote {
    ticker: String,
    price: i64,
    volume: u32,
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
    pub fn generate(ticker: &str) -> Self {
        let mut rng = rand::rng();
        Self {
            ticker: ticker.to_string(),
            price: rng.random_range(100..100000),
            volume: rng.random_range(10..1000),
            timestamp: Utc::now().timestamp(),
        }
    }

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
