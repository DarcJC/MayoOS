use core::sync::atomic::{AtomicU64, Ordering};

/// Process ID
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Eq, Ord)]
pub struct PID(u64);

impl PID {
    /// Create an new self-increasing pid
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
