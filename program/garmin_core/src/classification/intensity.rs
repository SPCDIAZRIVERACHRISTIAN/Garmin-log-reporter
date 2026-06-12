use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Intensity {
    Recovery,
    Easy,
    Moderate,
    Threshold,
    Vo2Max,
}
