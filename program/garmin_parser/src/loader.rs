use std::path::Path;
use walkdir::WalkDir;

use garmin_core::activity::Activity;

use crate::parser::parse_activity_file;

pub fn load_all_activities(base: &Path) -> Result<Vec<Activity>, Box<dyn std::error::Error>> {
    let mut activities = Vec::new();

    for entry in WalkDir::new(base) {
        let entry = entry?;

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let activity = parse_activity_file(path)?;

        activities.push(activity);
    }

    Ok(activities)
}
