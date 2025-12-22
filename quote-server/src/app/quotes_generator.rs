use crate::app::server_cancellation_token::ServerCancellationToken;
use crossbeam_channel::{Receiver, Sender, unbounded};
use quote_streaming::StockQuote;
use std::io::{BufRead, BufReader, Read};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use tracing::{error, instrument, trace};

#[instrument(name = "Run quotes generator", skip_all)]
pub(crate) fn run_quotes_generator<R: Read>(
    reader: R,
    cancellation_token: Arc<ServerCancellationToken>,
) -> Result<(Receiver<StockQuote>, JoinHandle<()>), std::io::Error> {
    let buffer = BufReader::new(reader);
    let mut tickers = Vec::new();
    for line in buffer.lines() {
        tickers.push(line?.trim().to_string());
    }

    let (tx, rx) = unbounded::<StockQuote>();
    let thread = thread::spawn(move || quotes_generator(tx, cancellation_token, tickers));

    Ok((rx, thread))
}

#[instrument(name = "Generate quotes", skip_all)]
fn quotes_generator(
    tx: Sender<StockQuote>,
    cancellation_token: Arc<ServerCancellationToken>,
    tickers: Vec<String>,
) {
    loop {
        if cancellation_token.is_cancelled() {
            return;
        }

        for ticker in tickers.iter() {
            let quote = StockQuote::generate(ticker);
            match tx.send(quote) {
                Ok(_) => trace!("Quote {} was generated and sent", ticker),
                Err(_) => {
                    error!("Failed to send quotes");
                    cancellation_token.cancel();
                    return;
                }
            }
        }

        thread::sleep(Duration::from_secs(3));
    }
}
