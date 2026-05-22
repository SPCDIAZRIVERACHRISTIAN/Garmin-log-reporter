use std::path::PathBuf;

use chrono::{DateTime, Datelike, Utc};
use garmin_core::activity::{Activity, ActivityId, ActivityMetadata, Split};
use garmin_core::classification::Terrain;
use garmin_core::metrics::HeartRate;
use garmin_core::units::{Distance, Duration, Pace, Timestamp};

pub fn save_in_file(activity: &Activity) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let date = activity.start_time.date();
    let year = date.year();
    let month = date.month();
    let id = activity.id.value();

    let dir = data_dir()?
        .join(format!("{:04}", year))
        .join(format!("{:02}", month));

    std::fs::create_dir_all(&dir)?;

    let json = build_json(activity);
    let file_path = dir.join(format!("{}.json", id));
    std::fs::write(&file_path, json)?;

    Ok(file_path)
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

fn parse_activity_json(content: &str) -> Result<Activity, Box<dyn std::error::Error>> {
    let v: serde_json::Value = serde_json::from_str(content)?;

    let id = ActivityId::new(v["id"].as_u64().ok_or("missing id")?);

    let terrain = match v["terrain"].as_str().ok_or("missing terrain")? {
        "Road" => Terrain::Road,
        "Trail" => Terrain::Trail,
        _ => Terrain::Unknown,
    };

    let start_time_str = v["start_time"].as_str().ok_or("missing start_time")?;
    let dt = DateTime::parse_from_rfc3339(start_time_str)?.with_timezone(&Utc);
    let start_time = Timestamp::new(dt);

    let distance = Distance::from_meters(v["distance_meters"].as_f64().ok_or("missing distance_meters")?);
    let duration = Duration::from_seconds(v["duration_seconds"].as_f64().ok_or("missing duration_seconds")?);

    let avg_hr = v["avg_hr_bpm"].as_u64().map(|b| HeartRate::new(b as u16));
    let max_hr = v["max_hr_bpm"].as_u64().map(|b| HeartRate::new(b as u16));

    let splits = v["splits"]
        .as_array()
        .ok_or("missing splits")?
        .iter()
        .map(|s| {
            let index = s["index"].as_u64().ok_or("missing split index")? as u32;
            let dist = Distance::from_meters(s["distance_meters"].as_f64().ok_or("missing split distance_meters")?);
            let dur = Duration::from_seconds(s["duration_seconds"].as_f64().ok_or("missing split duration_seconds")?);
            let avg_hr = s["avg_hr_bpm"].as_u64().map(|b| HeartRate::new(b as u16));
            let pace = s["pace_seconds_per_km"].as_f64().map(|sec| {
                Pace::new(Duration::from_seconds(sec), Distance::from_meters(1000.0))
            });
            Ok(Split::new(index, dist, dur, avg_hr, pace))
        })
        .collect::<Result<Vec<Split>, Box<dyn std::error::Error>>>()?;

    let metadata = ActivityMetadata {
        device: v["metadata"]["device"].as_str().map(String::from),
        notes: v["metadata"]["notes"].as_str().map(String::from),
        source: v["metadata"]["source"].as_str().map(String::from),
    };

    Ok(Activity::new(id, terrain, start_time, distance, duration, avg_hr, max_hr, splits, metadata))
}

fn build_json(activity: &Activity) -> String {
    let avg_hr = activity
        .avg_hr
        .map(|h| serde_json::Value::from(h.bpm()))
        .unwrap_or(serde_json::Value::Null);

    let max_hr = activity
        .max_hr
        .map(|h| serde_json::Value::from(h.bpm()))
        .unwrap_or(serde_json::Value::Null);

    let splits: Vec<serde_json::Value> = activity
        .splits
        .iter()
        .map(|s| {
            let split_avg_hr = s
                .avg_hr
                .map(|h| serde_json::Value::from(h.bpm()))
                .unwrap_or(serde_json::Value::Null);

            let split_pace = s
                .pace
                .map(|p| serde_json::Value::from(p.seconds_per_km()))
                .unwrap_or(serde_json::Value::Null);

            serde_json::json!({
                "index": s.index,
                "distance_meters": s.distance.meters(),
                "duration_seconds": s.duration.seconds(),
                "avg_hr_bpm": split_avg_hr,
                "pace_seconds_per_km": split_pace,
            })
        })
        .collect();

    let terrain = format!("{:?}", activity.terrain);

    let value = serde_json::json!({
        "id": activity.id.value(),
        "terrain": terrain,
        "start_time": activity.start_time.value().to_rfc3339(),
        "distance_meters": activity.distance.meters(),
        "duration_seconds": activity.duration.seconds(),
        "avg_hr_bpm": avg_hr,
        "max_hr_bpm": max_hr,
        "splits": splits,
        "metadata": {
            "device": activity.metadata.device,
            "notes": activity.metadata.notes,
            "source": activity.metadata.source,
        }
    });

    serde_json::to_string_pretty(&value).expect("serde_json::Value serialization is infallible")
}
