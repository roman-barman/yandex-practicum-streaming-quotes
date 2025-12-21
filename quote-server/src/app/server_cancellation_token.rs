use std::sync::atomic::AtomicBool;

#[derive(Default)]
pub(crate) struct ServerCancellationToken(AtomicBool);

impl ServerCancellationToken {
    pub(crate) fn cancel(&self) {
        self.0.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancel() {
        let token = ServerCancellationToken::default();
        token.cancel();
        assert!(token.is_cancelled());
    }
}
