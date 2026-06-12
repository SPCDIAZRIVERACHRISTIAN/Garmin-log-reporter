use std::fs;
use std::path::{Path, PathBuf};

use chrono::Datelike;
use garmin_core::activity::Activity;

use crate::error::StorageError;
use crate::paths::storage_root;

/// Result of saving a single activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveOutcome {
    Saved,
    SkippedExisting,
    Overwritten,
}

/// Aggregate result of saving a batch of activities.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SaveSummary {
    pub saved: usize,
    pub skipped: usize,
    pub failed: usize,
}

/// Save one activity under the default storage root.
///
/// Existing activities are skipped, never overwritten.
pub fn save_activity(activity: &Activity) -> Result<SaveOutcome, StorageError> {
    save_activity_in(&storage_root()?, activity)
}

/// Save one activity under an explicit data root (skips existing files).
pub fn save_activity_in(
    data_root: &Path,
    activity: &Activity,
) -> Result<SaveOutcome, StorageError> {
    save_activity_with(data_root, activity, false)
}

/// Save one activity under an explicit data root, replacing any existing file.
pub fn overwrite_activity_in(
    data_root: &Path,
    activity: &Activity,
) -> Result<SaveOutcome, StorageError> {
    save_activity_with(data_root, activity, true)
}

fn save_activity_with(
    data_root: &Path,
    activity: &Activity,
    overwrite: bool,
) -> Result<SaveOutcome, StorageError> {
    let file_path = activity_path(data_root, activity);
    let dir = file_path
        .parent()
        .expect("activity path always has a parent");
    fs::create_dir_all(dir).map_err(|e| StorageError::Io {
        path: dir.to_path_buf(),
        source: e,
    })?;

    let existed = file_path.exists();
    if existed && !overwrite {
        return Ok(SaveOutcome::SkippedExisting);
    }

    let json = serde_json::to_string_pretty(activity).map_err(|e| StorageError::Json {
        path: file_path.clone(),
        source: e,
    })?;
    fs::write(&file_path, json).map_err(|e| StorageError::Io {
        path: file_path.clone(),
        source: e,
    })?;

    Ok(if existed {
        SaveOutcome::Overwritten
    } else {
        SaveOutcome::Saved
    })
}

/// Save a batch of activities under the default storage root.
///
/// Per-activity failures are counted in the summary instead of aborting the batch.
pub fn save_activities(activities: &[Activity]) -> Result<SaveSummary, StorageError> {
    let root = storage_root()?;
    save_activities_in(&root, activities)
}

/// Save a batch of activities under an explicit data root.
pub fn save_activities_in(
    data_root: &Path,
    activities: &[Activity],
) -> Result<SaveSummary, StorageError> {
    let mut summary = SaveSummary::default();
    for activity in activities {
        match save_activity_in(data_root, activity) {
            Ok(SaveOutcome::Saved) | Ok(SaveOutcome::Overwritten) => summary.saved += 1,
            Ok(SaveOutcome::SkippedExisting) => summary.skipped += 1,
            Err(_) => summary.failed += 1,
        }
    }
    Ok(summary)
}

/// Load every stored activity from the default storage root.
///
/// A missing storage directory yields an empty vector, not an error.
pub fn load_activities() -> Result<Vec<Activity>, StorageError> {
    load_activities_in(&storage_root()?)
}

/// Load every stored activity under an explicit data root.
pub fn load_activities_in(data_root: &Path) -> Result<Vec<Activity>, StorageError> {
    let mut activities = Vec::new();
    for path in stored_json_files(data_root)? {
        activities.push(read_activity(&path)?);
    }
    Ok(activities)
}

/// Load one activity by id from the default storage root.
pub fn load_activity_by_id(id: u64) -> Result<Option<Activity>, StorageError> {
    load_activity_by_id_in(&storage_root()?, id)
}

/// Load one activity by id under an explicit data root.
pub fn load_activity_by_id_in(data_root: &Path, id: u64) -> Result<Option<Activity>, StorageError> {
    let target = format!("{id}.json");
    for path in stored_json_files(data_root)? {
        if path.file_name().and_then(|n| n.to_str()) == Some(target.as_str()) {
            return Ok(Some(read_activity(&path)?));
        }
    }
    Ok(None)
}

fn activity_path(data_root: &Path, activity: &Activity) -> PathBuf {
    let date = activity.start_time.date();
    data_root
        .join(format!("{:04}", date.year()))
        .join(format!("{:02}", date.month()))
        .join(format!("{}.json", activity.id.value()))
}

/// All `{year}/{month}/{id}.json` files under the data root, in directory order.
fn stored_json_files(data_root: &Path) -> Result<Vec<PathBuf>, StorageError> {
    let mut files = Vec::new();
    if !data_root.exists() {
        return Ok(files);
    }

    for year_entry in read_dir_sorted(data_root)? {
        if !year_entry.is_dir() {
            continue;
        }
        for month_entry in read_dir_sorted(&year_entry)? {
            if !month_entry.is_dir() {
                continue;
            }
            for file_entry in read_dir_sorted(&month_entry)? {
                if file_entry.extension().and_then(|e| e.to_str()) == Some("json") {
                    files.push(file_entry);
                }
            }
        }
    }

    Ok(files)
}

fn read_dir_sorted(dir: &Path) -> Result<Vec<PathBuf>, StorageError> {
    let entries = fs::read_dir(dir).map_err(|e| StorageError::Io {
        path: dir.to_path_buf(),
        source: e,
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| StorageError::Io {
            path: dir.to_path_buf(),
            source: e,
        })?;
        paths.push(entry.path());
    }
    paths.sort();
    Ok(paths)
}

fn read_activity(path: &Path) -> Result<Activity, StorageError> {
    let content = fs::read_to_string(path).map_err(|e| StorageError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    serde_json::from_str::<Activity>(&content).map_err(|e| StorageError::Json {
        path: path.to_path_buf(),
        source: e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use garmin_core::activity::{ActivityId, ActivityMetadata};
    use garmin_core::classification::Terrain;
    use garmin_core::metrics::HeartRate;
    use garmin_core::units::{Distance, Duration, Timestamp};

    fn sample_activity(id: u64, year: i32, month: u32, day: u32) -> Activity {
        let dt = Utc
            .with_ymd_and_hms(year, month, day, 6, 30, 0)
            .single()
            .unwrap();
        Activity::new(
            ActivityId::new(id),
            Terrain::Unknown,
            Timestamp::new(dt),
            Distance::from_meters(5000.0),
            Duration::from_seconds(1500.0),
            Some(HeartRate::new(150)),
            Some(HeartRate::new(170)),
            vec![],
            ActivityMetadata::empty(),
        )
    }

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("data");
        let activity = sample_activity(42, 2026, 6, 1);

        let outcome = save_activity_in(&root, &activity).unwrap();
        assert_eq!(outcome, SaveOutcome::Saved);

        let loaded = load_activities_in(&root).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id.value(), 42);
        assert_eq!(loaded[0].distance.meters(), 5000.0);
        assert_eq!(loaded[0].avg_hr.map(|h| h.bpm()), Some(150));
        assert_eq!(loaded[0].start_time, activity.start_time);
    }

    #[test]
    fn save_multiple_and_load_all() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("data");
        let activities = vec![
            sample_activity(1, 2026, 1, 10),
            sample_activity(2, 2026, 2, 11),
            sample_activity(3, 2025, 12, 25),
        ];

        let summary = save_activities_in(&root, &activities).unwrap();
        assert_eq!(summary.saved, 3);
        assert_eq!(summary.skipped, 0);
        assert_eq!(summary.failed, 0);

        let mut loaded = load_activities_in(&root).unwrap();
        loaded.sort_by_key(|a| a.id.value());
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].id.value(), 1);
        assert_eq!(loaded[2].id.value(), 3);
    }

    #[test]
    fn missing_storage_dir_returns_empty_vec() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("does-not-exist");
        let loaded = load_activities_in(&root).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn save_creates_year_month_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("data");
        let activity = sample_activity(7, 2026, 6, 12);

        save_activity_in(&root, &activity).unwrap();

        let expected = root.join("2026").join("06").join("7.json");
        assert!(expected.is_file());
    }

    #[test]
    fn saving_same_activity_twice_skips_second() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("data");
        let activity = sample_activity(9, 2026, 3, 3);

        assert_eq!(
            save_activity_in(&root, &activity).unwrap(),
            SaveOutcome::Saved
        );
        assert_eq!(
            save_activity_in(&root, &activity).unwrap(),
            SaveOutcome::SkippedExisting
        );

        let summary = save_activities_in(&root, std::slice::from_ref(&activity)).unwrap();
        assert_eq!(summary.skipped, 1);
        assert_eq!(summary.saved, 0);
    }

    #[test]
    fn overwrite_replaces_existing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("data");
        let activity = sample_activity(11, 2026, 4, 4);

        save_activity_in(&root, &activity).unwrap();
        assert_eq!(
            overwrite_activity_in(&root, &activity).unwrap(),
            SaveOutcome::Overwritten
        );
    }

    #[test]
    fn load_by_id_returns_correct_activity() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("data");
        save_activities_in(
            &root,
            &[
                sample_activity(100, 2026, 1, 1),
                sample_activity(200, 2026, 2, 2),
            ],
        )
        .unwrap();

        let found = load_activity_by_id_in(&root, 200).unwrap();
        assert_eq!(found.map(|a| a.id.value()), Some(200));
    }

    #[test]
    fn load_by_missing_id_returns_none() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("data");
        save_activity_in(&root, &sample_activity(100, 2026, 1, 1)).unwrap();

        let found = load_activity_by_id_in(&root, 999).unwrap();
        assert!(found.is_none());
    }
}
