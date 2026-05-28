use std::time::{Instant, Duration};

#[derive(Default)]
pub struct Timer {
    started_at: Option<Instant>,
    accumulated: Duration
}

impl Timer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self) {
        if self.started_at.is_none() {
            self.started_at = Some(Instant::now());
        }
    }

    pub fn stop(&mut self) {
        if let Some(start) = self.started_at.take() {
            self.accumulated += start.elapsed();
        }
    }

    pub fn reset(&mut self) {
        self.started_at = None;
        self.accumulated = Duration::ZERO;
    }

    pub fn elapsed(&self) -> Duration {
        match self.started_at {
            Some(start) => self.accumulated + start.elapsed(),
            None => self.accumulated
        }
    }
}
