use crate::clock::Clock;
use chrono::{DateTime, Utc};
pub enum Resolution {
    Minute,
    Day,
}

pub struct Market {
    resolution: Resolution,
    clock: Clock,
}

impl Market {
    pub fn new(resolution: Resolution, timestamps: Vec<DateTime<Utc>>) -> Self {
        let clock = Clock::new(timestamps);
        Self { resolution, clock }
    }
}
