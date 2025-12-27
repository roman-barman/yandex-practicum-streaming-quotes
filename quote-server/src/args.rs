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

    /// Log level
    #[arg(short = 'l', long, value_enum)]
    pub log_level: Option<LogLevel>,
}

impl Args {
    pub(super) fn log_level(&self) -> &str {
        self.log_level.as_ref().unwrap_or(&LogLevel::Info).as_str()
    }
}

#[derive(Debug, clap::ValueEnum, Clone)]
pub(super) enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}
