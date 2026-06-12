use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Distance(pub f64);

impl Distance {
    pub fn from_meters(m: f64) -> Self {
        assert!(m >= 0.0, "Distance is negative fix ASAP");
        Self(m)
    }

    pub fn meters(self) -> f64 {
        self.0
    }

    pub fn kilometers(self) -> f64 {
        self.0 / 1000.0
    }
}
