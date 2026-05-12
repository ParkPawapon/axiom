use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct ProcessTimeoutGuard {
    started_at: Instant,
    timeout: Duration,
}

impl ProcessTimeoutGuard {
    pub fn new(timeout: Duration) -> Self {
        Self {
            started_at: Instant::now(),
            timeout,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.elapsed().as_millis()
    }

    pub fn is_expired(&self) -> bool {
        self.elapsed() >= self.timeout
    }
}
