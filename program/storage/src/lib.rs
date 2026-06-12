pub mod error;
pub mod file;
pub mod paths;

pub use error::StorageError;
pub use file::{
    SaveOutcome, SaveSummary, load_activities, load_activities_in, load_activity_by_id,
    load_activity_by_id_in, overwrite_activity_in, save_activities, save_activities_in,
    save_activity, save_activity_in,
};
pub use paths::{
    DATA_DIR_ENV, app_root, data_root_in, raw_root, raw_root_in, resolve_app_root, storage_root,
};
