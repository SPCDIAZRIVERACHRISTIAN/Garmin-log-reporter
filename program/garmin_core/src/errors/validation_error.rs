use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ValidationError {
    NegativeDistance(f64),
    NoPace(f64),
    AbsurdHeartRate(u16),
    Inconsistency(&'static str),
}

impl Error for ValidationError {}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::NegativeDistance(distance) => {
                write!(f, "Negative distance given: {}", distance)
            }
            ValidationError::NoPace(pace) => {
                write!(f, "No pace marked: {}", pace)
            }
            ValidationError::AbsurdHeartRate(hr) => {
                write!(
                    f,
                    "You are either dead or have a messed up garmin look at the hr: {}",
                    hr
                )
            }
            ValidationError::Inconsistency(inconst) => {
                write!(f, "Inconsistency detected: {}", inconst)
            }
        }
    }
}
