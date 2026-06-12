use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum StorageError {
    /// No usable data directory: platform dir missing and GARMIN_LOG_DATA_DIR unset.
    DataDirUnavailable,
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::DataDirUnavailable => write!(
                f,
                "could not resolve a local data directory; set GARMIN_LOG_DATA_DIR"
            ),
            StorageError::Io { path, source } => {
                write!(f, "io error at {}: {}", path.display(), source)
            }
            StorageError::Json { path, source } => {
                write!(f, "invalid activity json at {}: {}", path.display(), source)
            }
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::DataDirUnavailable => None,
            StorageError::Io { source, .. } => Some(source),
            StorageError::Json { source, .. } => Some(source),
        }
    }
}
