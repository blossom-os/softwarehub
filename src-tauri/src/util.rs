use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct SpeedCalculator {
    last_bytes: Arc<Mutex<u64>>,
    last_time: Arc<Mutex<Instant>>,
}

impl SpeedCalculator {
    pub fn new() -> Self {
        Self {
            last_bytes: Arc::new(Mutex::new(0)),
            last_time: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn calculate_speed(&self, bytes: u64) -> f64 {
        let now = Instant::now();
        let mut last_bytes_guard = self.last_bytes.lock().unwrap();
        let mut last_time_guard = self.last_time.lock().unwrap();
        let elapsed = now.duration_since(*last_time_guard).as_secs_f64();

        let speed_mbps = if elapsed > 0.1 && bytes > *last_bytes_guard {
            let bytes_diff = bytes - *last_bytes_guard;
            (bytes_diff as f64 / elapsed) / 1_000_000.0 // Convert to MB/s
        } else {
            0.0
        };

        if elapsed > 0.1 {
            *last_bytes_guard = bytes;
            *last_time_guard = now;
        }

        speed_mbps
    }
}

