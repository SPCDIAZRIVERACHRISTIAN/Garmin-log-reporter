use garmin_core::activity::Activity;
use garmin_core::activity::{ActivityId, ActivityMetadata, Split};
use garmin_core::classification::Terrain;
use garmin_core::metrics::HeartRate;
use garmin_core::units::{Distance, Duration, Timestamp};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawActivity {
    pub activity_id: u64,
    pub activity_name: String,
    pub sport: String,
    pub start_time: String,
    pub timestamp: u64,
    pub distance_meters: f64,
    pub duration_seconds: f64,
    pub avg_hr: Option<f64>,
    pub max_hr: Option<f64>,
    pub avg_speed: Option<f64>,
    pub max_speed: Option<f64>,
    pub calories: Option<f64>,
    pub steps: Option<u64>,
    pub location: Option<String>,
}

impl TryFrom<RawActivity> for Activity {
    type Error = String;

    fn try_from(raw: RawActivity) -> Result<Self, Self::Error> {
        let terrain = Terrain::from_str(&raw.sport).map_err(|_| "invalid terrain")?;

        Ok(Self {
            id: ActivityId::new(raw.activity_id),
            terrain,
            start_time: Timestamp::from_epoch(raw.timestamp),
            distance: Distance::meters(raw.distance_meters),
            duration: Duration::seconds(raw.duration_seconds),
            avg_hr: raw.avg_hr.map(HeartRate::new),
            max_hr: raw.max_hr.map(HeartRate::new),
            splits: Vec::new(),
            metadata: ActivityMetadata::from_raw(&raw),
        })
    }
}
