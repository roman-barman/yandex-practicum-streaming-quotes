use std::sync::atomic::AtomicBool;

#[derive(Default)]
pub(super) struct CancellationToken(AtomicBool);

impl CancellationToken {
    pub(super) fn cancel(&self) {
        self.0.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub(super) fn is_cancelled(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}
