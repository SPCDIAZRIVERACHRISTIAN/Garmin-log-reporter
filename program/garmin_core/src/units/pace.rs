use super::Distance;
use super::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Pace {
    duration: Duration,
    distance: Distance,
}

impl Pace {
    pub fn new(duration: Duration, distance: Distance) -> Self {
        Self { duration, distance }
    }

    pub fn seconds_per_km(self) -> f64 {
        self.duration.seconds() / self.distance.kilometers()
    }
}
