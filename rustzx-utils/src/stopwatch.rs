use rustzx_core::host::Stopwatch;
use std::time::{Duration, Instant};

pub struct InstantStopwatch {
    timestamp: Instant,
}

impl Default for InstantStopwatch {
    fn default() -> Self {
        Self {
            timestamp: Instant::now(),
        }
    }
}

impl Stopwatch for InstantStopwatch {
    fn new() -> Self {
        Self::default()
    }

    fn measure(&self) -> Duration {
        self.timestamp.elapsed()
    }
}
