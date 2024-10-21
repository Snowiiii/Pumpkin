use std::time::{Duration, Instant};

use super::Server;

pub struct Ticker {
    tick_interval: Duration,
    last_tick: Instant,
}

impl Ticker {
    pub fn new(tps: f32) -> Self {
        Self {
            tick_interval: Duration::from_millis((1000.0 / tps) as u64),
            last_tick: Instant::now(),
        }
    }

    /// IMPORTANT: Run this in a new thread/tokio task
    pub async fn run(&mut self, server: &Server) {
        loop {
            let now = Instant::now();
            let elapsed = now - self.last_tick;

            if elapsed >= self.tick_interval {
                server.tick().await;
                self.last_tick = now;
            } else {
                // Wait for the remaining time until the next tick
                let sleep_time = self.tick_interval - elapsed;
                std::thread::sleep(sleep_time);
            }
        }
    }
}
