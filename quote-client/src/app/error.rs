use crate::app::read_udp_response::ReadUdpResponseError;
use crate::app::server_connect::ServerConnectError;

#[derive(Debug, thiserror::Error)]
pub(crate) enum AppError {
    #[error("Error setting Ctrl-C handler")]
    Ctrlc(#[from] ctrlc::Error),
    #[error("Failed to create UDP socket")]
    UdpSocket(#[from] std::io::Error),
    #[error("Failed to connect to server: {0}")]
    ServerConnect(#[from] ServerConnectError),
    #[error("Failed to read UDP response: {0}")]
    ReadResponse(#[from] ReadUdpResponseError),
    #[error("Failed to join thread: {0}")]
    JoinThread(String),
}
