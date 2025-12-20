use std::net::IpAddr;

#[derive(Debug, clap::Parser)]
pub(super) struct Args {
    /// Path to the file with tickers
    #[clap(short = 't', long)]
    pub tickers_file: std::path::PathBuf,

    /// Port to listen on
    #[clap(short = 'p', long)]
    pub port: Option<u16>,

    /// Address to listen on
    #[clap(short = 'a', long)]
    pub address: Option<IpAddr>,
}
