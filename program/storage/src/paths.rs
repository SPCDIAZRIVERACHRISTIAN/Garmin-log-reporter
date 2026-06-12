use std::env;
use std::path::{Path, PathBuf};

use crate::error::StorageError;

/// Environment variable that overrides the application data root.
pub const DATA_DIR_ENV: &str = "GARMIN_LOG_DATA_DIR";

/// Application data root.
///
/// Resolution order:
/// 1. `GARMIN_LOG_DATA_DIR` environment variable
/// 2. `~/.local/share/garmin-log-reporter` (platform local data dir)
pub fn app_root() -> Result<PathBuf, StorageError> {
    resolve_app_root(None)
}

/// Application data root with an explicit override (e.g. a `--data-dir` CLI flag).
///
/// The override wins over the environment variable, which wins over the
/// platform default.
pub fn resolve_app_root(override_dir: Option<&Path>) -> Result<PathBuf, StorageError> {
    if let Some(dir) = override_dir {
        return Ok(dir.to_path_buf());
    }
    if let Some(dir) = env::var_os(DATA_DIR_ENV)
        && !dir.is_empty()
    {
        return Ok(PathBuf::from(dir));
    }
    dirs::data_local_dir()
        .map(|d| d.join("garmin-log-reporter"))
        .ok_or(StorageError::DataDirUnavailable)
}

/// Where normalized activities live: `<app_root>/data/{year}/{month}/{id}.json`.
pub fn storage_root() -> Result<PathBuf, StorageError> {
    Ok(app_root()?.join("data"))
}

/// Normalized-activity root under an explicit app root.
pub fn data_root_in(app_root: &Path) -> PathBuf {
    app_root.join("data")
}

/// Where raw fetched Garmin JSON lives: `<app_root>/raw/{id}.json`.
pub fn raw_root() -> Result<PathBuf, StorageError> {
    Ok(app_root()?.join("raw"))
}

/// Raw-JSON root under an explicit app root.
pub fn raw_root_in(app_root: &Path) -> PathBuf {
    app_root.join("raw")
}
