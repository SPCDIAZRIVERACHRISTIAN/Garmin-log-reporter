use serde:: {Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Duration(pub f64);

impl Duration {
    pub fn from_seconds(sec: f64) -> Self {
        assert!(sec > 0.0, "Duration must be positive");
        Self(sec)
    }

    pub fn seconds(self) -> f64 {
        self.0
    }

    pub fn minutes(self) -> f64 {
        self.0 / 60.0
    }
}
