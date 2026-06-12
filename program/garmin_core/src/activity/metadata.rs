use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityMetadata {
    pub device: Option<String>,
    pub notes: Option<String>,
    pub source: Option<String>,
}

impl ActivityMetadata {
    pub fn empty() -> Self {
        Self {
            device: None,
            notes: None,
            source: None,
        }
    }
}
