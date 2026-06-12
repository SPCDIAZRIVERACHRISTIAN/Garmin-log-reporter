use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TrainingEffect(f32);

impl TrainingEffect {
    pub fn new(score: f32) -> Self {
        assert!((0.0..=5.0).contains(&score), "Invalid training effect");
        Self(score)
    }

    pub fn value(self) -> f32 {
        self.0
    }
}
