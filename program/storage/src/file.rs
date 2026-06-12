use std::path::Path;
use std::fs;
use dirs::data_local_dir;
use chrono::{DateTime, Datelike, Utc};
use garmin_core::activity::{Activity, ActivityId, ActivityMetadata, Split};
use garmin_core::classification::Terrain;
use garmin_core::metrics::HeartRate;
use garmin_core::units::{Distance, Duration, Pace, Timestamp};

pub fn save_in_file(activity: &Activity) -> Result<(), Box<dyn std::error::Error>> {
    let path = data_local_dir().ok_or("local dir not found")?; // NOTE make a storage error struct to
                                                               //handle this type of error
    let date = activity.timestamp.date();
    let year = date.year();
    let month = date.month();
    let id = activity.id.value();

    let app_data = path
       .join("Garmin/data/")
       .join(format!("{:04}", year))
       .join(format!("{:02}", month));
 
    fs::create_dir_all(&app_data)?;
    println!("Directory ensured at: {}", app_data.display());
    
    let file_path = app_data.join(format!("{}.json", id));
    if !Path::new(&file_path).exists() {
        let json = serde_json::to_string_pretty(&activity)?;
        std::fs::write(&file_path, json)?;
        println!("file does not exists");
    }
    Ok(())
}

pub fn load_from_file() -> Result<Vec<Activity>, Box<dyn std::error::Error>> {
    let base = data_dir()?;

    if !base.exists() {
        return Ok(Vec::new());
    }

    let mut activities = Vec::new();

    for year_entry in std::fs::read_dir(&base)? {
        let year_entry = year_entry?;
        if !year_entry.file_type()?.is_dir() {
            continue;
        }

        for month_entry in std::fs::read_dir(year_entry.path())? {
            let month_entry = month_entry?;
            if !month_entry.file_type()?.is_dir() {
                continue;
            }

            for file_entry in std::fs::read_dir(month_entry.path())? {
                let file_entry = file_entry?;
                let path = file_entry.path();

                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }

                let content = std::fs::read_to_string(&path)?;
                let activity = parse_activity_json(&content)?;
                activities.push(activity);
            }
        }
    }

    Ok(activities)
}

fn data_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = dirs::data_local_dir()
        .ok_or("could not resolve local data directory")?
        .join("garmin-log-reporter")
        .join("activities");
    Ok(dir)
}
