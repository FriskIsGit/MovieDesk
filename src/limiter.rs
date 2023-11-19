use std::time::{Duration, Instant};

pub struct RateLimiter {
    pub max_hits: usize,
    pub time_frame: Duration,
    hits: Vec<Instant>,
}

impl RateLimiter {
    pub fn new(max_hits: usize, time_frame: Duration) -> Self {
        Self {
            max_hits,
            time_frame,
            hits: Vec::with_capacity(max_hits),
        }
    }

    pub fn hit(&mut self) -> bool {
        let now = Instant::now();

        let mut last_index = 0;
        for (i, time_stamp) in self.hits.iter().enumerate() {
            if now.duration_since(*time_stamp) > self.time_frame {
                last_index = i;
            } else {
                break;
            }
        }

        if last_index > 0 {
            self.hits.drain(0..=last_index);
        }

        if self.hits.len() > self.max_hits {
            return false;
        }

        self.hits.push(now);
        true
    }
}
