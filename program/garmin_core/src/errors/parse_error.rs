use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ParseError {
    MissingField(&'static str),
    InvalidNumber(String),
    InvalidTimeStamp(String),
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingField(field) => {
                write!(f, "Missing requiered field: {}", field)
            }
            ParseError::InvalidNumber(val) => {
                write!(f, "Invalid number format: {}", val)
            }
            ParseError::InvalidTimeStamp(ts) => {
                write!(f, "Invalid timestamp: {}", ts)
            }
        }
    }
}
