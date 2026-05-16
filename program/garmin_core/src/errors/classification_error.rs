use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ClassError {
    UndeterminedIntensity(String),
    UninferedTerrain(String),
    MissingMetrics(String),
}

impl fmt::Display for ClassError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClassError::UndeterminedIntensity(intensity) => {
                write!(f, "Undetermined intensity: {}", intensity)
            }
            ClassError::UninferedTerrain(terrain) => {
                write!(f, "Uninfered terrain: {}", terrain)
            }
            ClassError::MissingMetrics(metrics) => {
                write!(f, "Missing required metrics: {}", metrics)
            }
        }
    }
}

impl Error for ClassError {}
