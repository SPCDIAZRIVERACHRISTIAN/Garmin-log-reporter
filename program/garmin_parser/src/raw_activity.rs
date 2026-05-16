use chrono::NaiveDateTime;
use serde::Deserialize;
use garmin_core::activity::{Activity, ActivityId, ActivityMetadata};
use garmin_core::classification::Terrain;
use garmin_core::metrics::HeartRate;
use garmin_core::units::{Distance, Duration, Timestamp};

#[derive(Debug, Deserialize)]
pub struct RawActivity {
    pub activity_id: u64,
    pub activity_name: String,
    pub sport: String,
    pub start_time: String,
    pub timestamp: i64,

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

impl From<RawActivity> for Activity {
    fn from(raw: RawActivity) -> Self {
        let id = ActivityId::new(raw.activity_id);
        let terrain = Terrain::Unknown;
        let ndt = NaiveDateTime::parse_from_str(&raw.start_time, "%Y-%m-%d %H:%M:%S")
            .expect("invalid Garmin datetime string");
        let start_time = Timestamp::new(ndt.and_utc());
        let distance = Distance::from_meters(raw.distance_meters);
        let duration = Duration::from_seconds(raw.duration_seconds);
        let avg_hr = raw.avg_hr.map(|v| HeartRate::new(v as u16));
        let max_hr = raw.max_hr.map(|v| HeartRate::new(v as u16));
        let splits = vec![];
        let metadata = ActivityMetadata::empty();

        Activity::new(id, terrain, start_time, distance, duration, avg_hr, max_hr, splits, metadata)
    }
}
