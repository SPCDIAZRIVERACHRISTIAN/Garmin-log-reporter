use super::Distance;
use super::Duration;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
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
