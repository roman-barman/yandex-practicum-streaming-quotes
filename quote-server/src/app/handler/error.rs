#[derive(Debug, thiserror::Error)]
pub(crate) enum HandlerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Deserialization command error: {0}")]
    DeserializationOrSerialization(#[from] rancor::Error),
}
