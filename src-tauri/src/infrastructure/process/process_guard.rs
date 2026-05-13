use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
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

    pub fn is_expired(&self) -> bool {
        self.started_at.elapsed() >= self.timeout
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.started_at.elapsed().as_millis()
    }
}
