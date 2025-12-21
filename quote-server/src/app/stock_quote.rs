use chrono::Utc;
use rand::Rng;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct StockQuote {
    pub ticker: String,
    pub price: Decimal,
    pub volume: u32,
    pub timestamp: i64,
}

impl StockQuote {
    pub(super) fn generate(ticker: &str) -> Self {
        let mut rng = rand::rng();
        Self {
            ticker: ticker.to_string(),
            price: Decimal::new(rng.random_range(100..100000), 2),
            volume: rng.random_range(10..1000),
            timestamp: Utc::now().timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_quote() {
        let quote_1 = StockQuote::generate("AAPL");
        assert_eq!(quote_1.ticker, "AAPL");
        assert!(quote_1.price >= Decimal::new(100, 2));
        assert!(quote_1.volume >= 10);
        assert!(quote_1.timestamp > 0);

        let quote_2 = StockQuote::generate("MSFT");
        assert_ne!(quote_1.ticker, quote_2.ticker);
    }
}
