use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Calories(u32);

impl Calories {
    pub fn new(kcal: u32) -> Self {
        Self(kcal)
    }

    pub fn value(self) -> u32 {
        self.0
    }
}
