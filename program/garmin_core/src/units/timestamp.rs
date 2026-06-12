use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn new(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    pub fn value(self) -> DateTime<Utc> {
        self.0
    }

    pub fn from_epoch(ms: u64) -> Self {
        let seconds = (ms / 1000) as i64;
        let nanos = ((ms % 1000) * 1_000_000) as u32;

        let dt = Utc
            .timestamp_opt(seconds, nanos)
            .single()
            .expect("invalid timestamp");

        Self(dt)
    }

    pub fn date(&self) -> chrono::NaiveDate {
        self.0.date_naive()
    }
}
