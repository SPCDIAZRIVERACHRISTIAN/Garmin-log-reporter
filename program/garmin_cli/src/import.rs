use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};

use garmin_core::activity::Activity;
use garmin_parser::parser::parse_activity_file;
use storage::SaveOutcome;
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct ImportStats {
    pub imported: usize,
    pub skipped: usize,
    pub failed: usize,
    pub failures: Vec<(PathBuf, String)>,
}

/// Collect importable JSON files from a single file or a directory tree.
pub fn collect_json_files(path: &Path) -> Result<Vec<PathBuf>, String> {
    if !path.exists() {
        return Err(format!("Import path does not exist: {}", path.display()));
    }

    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry.map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        if entry.file_type().is_file()
            && entry.path().extension().and_then(|e| e.to_str()) == Some("json")
        {
            files.push(entry.into_path());
        }
    }
    files.sort();

    if files.is_empty() {
        return Err(format!("No Garmin JSON files found in: {}", path.display()));
    }
    Ok(files)
}

/// Parse and save each file, continuing past per-file failures.
pub fn import_files(files: &[PathBuf], data_root: &Path) -> ImportStats {
    let mut stats = ImportStats::default();

    // The parser asserts on invalid values (bad dates, zero durations), which
    // panics. Silence the default panic printer for the import loop so one bad
    // file becomes a counted failure instead of noise + crash.
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    for file in files {
        match parse_file(file) {
            Ok(activity) => match storage::save_activity_in(data_root, &activity) {
                Ok(SaveOutcome::Saved) | Ok(SaveOutcome::Overwritten) => stats.imported += 1,
                Ok(SaveOutcome::SkippedExisting) => stats.skipped += 1,
                Err(e) => {
                    stats.failed += 1;
                    stats.failures.push((file.clone(), e.to_string()));
                }
            },
            Err(message) => {
                stats.failed += 1;
                stats.failures.push((file.clone(), message));
            }
        }
    }

    panic::set_hook(previous_hook);
    stats
}

fn parse_file(path: &Path) -> Result<Activity, String> {
    match panic::catch_unwind(AssertUnwindSafe(|| parse_activity_file(path))) {
        Ok(Ok(activity)) => Ok(activity),
        Ok(Err(e)) => Err(e.to_string()),
        Err(payload) => Err(panic_message(payload)),
    }
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "parser panicked on invalid data".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn raw_activity_json(id: u64, start_time: &str) -> String {
        format!(
            r#"{{
    "activity_id": {id},
    "activity_name": "Test Run",
    "sport": "running",
    "start_time": "{start_time}",
    "timestamp": 1768651210000,
    "distance_meters": 3841.0,
    "duration_seconds": 1749.0,
    "avg_hr": 174.0,
    "max_hr": 188.0,
    "avg_speed": 2.19,
    "max_speed": 3.36,
    "calories": 370.0,
    "steps": 4534,
    "location": "Testville"
}}"#
        )
    }

    #[test]
    fn imports_valid_files_and_counts_failures() {
        let tmp = tempfile::tempdir().unwrap();
        let raw = tmp.path().join("raw");
        let data = tmp.path().join("data");
        fs::create_dir_all(&raw).unwrap();

        fs::write(
            raw.join("1.json"),
            raw_activity_json(1, "2026-01-17 08:00:10"),
        )
        .unwrap();
        fs::write(
            raw.join("2.json"),
            raw_activity_json(2, "2026-02-03 06:15:00"),
        )
        .unwrap();
        fs::write(raw.join("broken.json"), "{ not json").unwrap();
        // valid JSON but a date the parser cannot handle -> parser panic path
        fs::write(raw.join("3.json"), raw_activity_json(3, "not-a-date")).unwrap();

        let files = collect_json_files(&raw).unwrap();
        assert_eq!(files.len(), 4);

        let stats = import_files(&files, &data);
        assert_eq!(stats.imported, 2);
        assert_eq!(stats.skipped, 0);
        assert_eq!(stats.failed, 2);

        // re-import skips everything already stored
        let again = import_files(&files, &data);
        assert_eq!(again.imported, 0);
        assert_eq!(again.skipped, 2);
        assert_eq!(again.failed, 2);

        let stored = storage::load_activities_in(&data).unwrap();
        assert_eq!(stored.len(), 2);
    }

    #[test]
    fn missing_path_is_a_friendly_error() {
        let err = collect_json_files(Path::new("/definitely/not/here")).unwrap_err();
        assert!(err.starts_with("Import path does not exist:"));
    }

    #[test]
    fn empty_directory_is_a_friendly_error() {
        let tmp = tempfile::tempdir().unwrap();
        let err = collect_json_files(tmp.path()).unwrap_err();
        assert!(err.starts_with("No Garmin JSON files found in:"));
    }
}
