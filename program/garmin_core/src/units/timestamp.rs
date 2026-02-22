use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn new(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    pub fn value(self) -> DateTime<Utc> {
        self.0
    }
}
