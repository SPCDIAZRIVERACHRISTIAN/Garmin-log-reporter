use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::Deserialize;

/// Machine-readable summary the Python fetcher prints on stdout.
#[derive(Debug, Deserialize)]
pub struct FetchSummary {
    /// Part of the fetcher contract; Rust resolves its own raw dir and passes
    /// it to the fetcher, so this is only read in tests.
    #[allow(dead_code)]
    pub raw_dir: String,
    pub fetched: u64,
    pub new_files: u64,
    pub existing_files: u64,
    pub failed: u64,
    /// Also contract-only: sync imports everything in the raw dir instead of
    /// trusting this list, so `--skip-fetch` shares the same code path.
    #[allow(dead_code)]
    #[serde(default)]
    pub files: Vec<String>,
}

pub fn parse_fetch_summary(stdout: &str) -> Result<FetchSummary, String> {
    serde_json::from_str(stdout)
        .map_err(|e| format!("Could not parse the fetcher summary: {e}\nFetcher output:\n{stdout}"))
}

const PYTHON_MISSING: &str = "Python is required for Garmin sync.\n\
Install Python 3 and fetcher dependencies, or use:\n  garmin-log import <path>";

/// Run the Python fetcher, streaming its progress (stderr) to the terminal.
pub fn run_fetcher(
    raw_dir: &Path,
    limit: Option<u32>,
    days: Option<u32>,
) -> Result<FetchSummary, String> {
    let python = find_python().ok_or_else(|| PYTHON_MISSING.to_string())?;
    let script = fetcher_script()?;

    let mut cmd = Command::new(&python);
    cmd.arg(&script)
        .arg("--raw-dir")
        .arg(raw_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    if let Some(limit) = limit {
        cmd.args(["--limit", &limit.to_string()]);
    }
    if let Some(days) = days {
        cmd.args(["--days", &days.to_string()]);
    }

    let output = cmd.output().map_err(|e| {
        format!(
            "Failed to run the Garmin fetcher ({}): {e}",
            python.display()
        )
    })?;

    if !output.status.success() {
        // The fetcher already printed a detailed message on stderr.
        return Err(format!(
            "Garmin fetch failed (exit code {}). See the message above.\n\
             You can still import previously downloaded files:\n  garmin-log import <path>",
            output.status.code().unwrap_or(-1)
        ));
    }

    parse_fetch_summary(&String::from_utf8_lossy(&output.stdout))
}

/// Python interpreter, in priority order: $GARMIN_PYTHON, the fetcher's
/// virtualenv, python3, python.
fn find_python() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(custom) = std::env::var("GARMIN_PYTHON") {
        candidates.push(PathBuf::from(custom));
    }
    if let Ok(dir) = fetcher_dir() {
        candidates.push(dir.join(".venv/bin/python"));
    }
    candidates.push(PathBuf::from("python3"));
    candidates.push(PathBuf::from("python"));

    candidates.into_iter().find(|p| {
        Command::new(p)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    })
}

fn fetcher_dir() -> Result<PathBuf, String> {
    // Resolved relative to this crate, which works for `cargo run` from the
    // workspace. Set GARMIN_FETCHER to point at the script anywhere else.
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fetcher"))
}

fn fetcher_script() -> Result<PathBuf, String> {
    if let Ok(custom) = std::env::var("GARMIN_FETCHER") {
        return Ok(PathBuf::from(custom));
    }
    let script = fetcher_dir()?.join("garmin_fetcher.py");
    if script.is_file() {
        Ok(script)
    } else {
        Err(format!(
            "Could not find the Garmin fetcher script at {}.\n\
             Set GARMIN_FETCHER to its location, or use:\n  garmin-log import <path>",
            script.display()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fetcher_summary_json() {
        let json = r#"{
            "raw_dir": "/home/user/.local/share/garmin-log-reporter/raw",
            "fetched": 20,
            "new_files": 3,
            "existing_files": 17,
            "failed": 0,
            "files": ["/home/user/.local/share/garmin-log-reporter/raw/21576019816.json"]
        }"#;
        let summary = parse_fetch_summary(json).unwrap();
        assert_eq!(summary.fetched, 20);
        assert_eq!(summary.new_files, 3);
        assert_eq!(summary.existing_files, 17);
        assert_eq!(summary.failed, 0);
        assert_eq!(summary.files.len(), 1);
    }

    #[test]
    fn missing_files_field_defaults_to_empty() {
        let json = r#"{"raw_dir": "/tmp/raw", "fetched": 0, "new_files": 0, "existing_files": 0, "failed": 0}"#;
        let summary = parse_fetch_summary(json).unwrap();
        assert!(summary.files.is_empty());
    }

    #[test]
    fn garbage_summary_is_a_friendly_error() {
        let err = parse_fetch_summary("not json at all").unwrap_err();
        assert!(err.starts_with("Could not parse the fetcher summary:"));
    }
}
