use serde::{ Serialize, Deserialize };
use crate::metrics::HeartRate;
use crate::units::{Distance, Duration, Pace};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Split {
    pub index: u32,
    pub distance: Distance,
    pub duration: Duration,
    pub avg_hr: Option<HeartRate>,
    pub pace: Option<Pace>,
}

impl Split {
    pub fn new(
        index: u32,
        distance: Distance,
        duration: Duration,
        avg_hr: Option<HeartRate>,
        pace: Option<Pace>,
    ) -> Self {
        Self {
            index,
            distance,
            duration,
            avg_hr,
            pace,
        }
    }
}
