use crate::activity::{ActivityId, ActivityMetadata, Split};
use crate::classification::Terrain;
use crate::metrics::HeartRate;
use crate::units::{Distance, Duration, Timestamp};
use garmin_parser::raw_activity::RawActivity;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct Activity {
    pub id: ActivityId,
    pub terrain: Terrain,
    pub start_time: Timestamp,
    pub distance: Distance,
    pub duration: Duration,
    pub avg_hr: Option<HeartRate>,
    pub max_hr: Option<HeartRate>,
    pub splits: Vec<Split>,
    pub metadata: ActivityMetadata,
}

impl Activity {
    pub fn new(
        id: ActivityId,
        terrain: Terrain,
        start_time: Timestamp,
        distance: Distance,
        duration: Duration,
        avg_hr: Option<HeartRate>,
        max_hr: Option<HeartRate>,
        splits: Vec<Split>,
        metadata: ActivityMetadata,
    ) -> Self {
        Self {
            id,
            terrain,
            start_time,
            distance,
            duration,
            avg_hr,
            max_hr,
            splits,
            metadata,
        }
    }
}
