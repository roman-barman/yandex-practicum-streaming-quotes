use clap::Subcommand;
use std::net::IpAddr;

#[derive(Debug, clap::Parser)]
pub(super) struct Args {
    /// Tickers source
    #[command(subcommand)]
    pub tickers: TickersSource,

    /// Server port
    #[clap(short = 'p', long)]
    pub server_port: u16,

    /// Server address
    #[clap(short = 'a', long)]
    pub server_address: IpAddr,

    ///Client UDP port
    #[clap(short = 'u', long)]
    pub udp_port: Option<u16>,
}

#[derive(Debug, Subcommand, Clone)]
pub(super) enum TickersSource {
    /// File with tickers
    File {
        /// Path to the file with tickers
        #[arg(short = 'f', long)]
        tickers_file: std::path::PathBuf,
    },
    /// Tickers provided as command line arguments
    Args {
        #[arg(short = 't', long)]
        tickers: Vec<String>,
    },
}
