use crate::app::monitoring_router::MonitoringRouterError;
use crate::app::tickers_router::TickersRouterError;

#[derive(Debug, thiserror::Error)]
pub(super) enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Ctrl+C error: {0}")]
    Ctrc(#[from] ctrlc::Error),
    #[error("Thread join error")]
    ThreadJoin,
    #[error("Monitoring route error: {0}")]
    MonitoringRoute(#[from] MonitoringRouterError),
    #[error("Tickers route error: {0}")]
    TickersRoute(#[from] TickersRouterError),
}
