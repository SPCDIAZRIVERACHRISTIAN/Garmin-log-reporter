use std::fs;
use std::path::Path;

use crate::raw_activity::RawActivity;

pub fn parse_activity_file(path: &Path) -> Result<RawActivity, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;

    let activity: RawActivity = serde_json::from_str(&contents)?;

    Ok(activity)
}
