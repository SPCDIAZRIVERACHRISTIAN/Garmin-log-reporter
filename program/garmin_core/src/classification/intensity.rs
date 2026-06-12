use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Intensity {
    Recovery,
    Easy,
    Moderate,
    Threshold,
    Vo2Max,
}
