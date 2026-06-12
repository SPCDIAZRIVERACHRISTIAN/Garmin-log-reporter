use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Elevation(f64);

impl Elevation {
    pub fn meters(m: f64) -> Self {
        Self(m)
    }

    pub fn value(self) -> f64 {
        self.0
    }
}
