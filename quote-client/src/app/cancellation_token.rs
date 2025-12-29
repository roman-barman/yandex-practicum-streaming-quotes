use std::sync::atomic::AtomicBool;

#[derive(Default)]
pub(crate) struct CancellationToken(AtomicBool);

impl CancellationToken {
    pub(crate) fn cancel(&self) {
        self.0.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}
