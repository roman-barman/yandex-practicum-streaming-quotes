use chrono::Utc;
use rand::Rng;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
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
