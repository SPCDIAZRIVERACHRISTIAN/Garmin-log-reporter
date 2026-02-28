use std::{error::Error, fmt};

use crate::errors::classification_error::ClassError;
use crate::errors::parse_error::ParseError;
use crate::errors::validation_error::ValidationError;

#[derive(Debug)]
pub enum CoreError {
    Parse(ParseError),
    Validation(ValidationError),
    Classification(ClassError),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::Parse(e) => write!(f, "Parse error: {}", e),
            CoreError::Validation(e) => write!(f, "Validation error: {}", e),
            CoreError::Classification(e) => write!(f, "Classification error: {}", e),
        }
    }
}

impl Error for CoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CoreError::Parse(e) => Some(e),
            CoreError::Validation(e) => Some(e),
            CoreError::Classification(e) => Some(e),
        }
    }
}
