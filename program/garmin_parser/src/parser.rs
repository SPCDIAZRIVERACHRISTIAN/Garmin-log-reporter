use crate::raw_activity::RawActivity;
use garmin_core::activity::Activity;
use std::fs;
use std::path::Path;

pub fn parse_activity_file(path: &Path) -> Result<Activity, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;

    let raw_activity: RawActivity = serde_json::from_str(&contents)?;

    let activity: Activity = raw_activity.into();

    Ok(activity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::load_all_activities;
    use chrono::NaiveDate;

    fn activity_path(relative: &str) -> std::path::PathBuf {
        let manifest = env!("CARGO_MANIFEST_DIR");
        Path::new(manifest)
            .join("../../activity/activities")
            .join(relative)
    }

    #[test]
    fn parses_all_fields_correctly() {
        let path = activity_path("2026/01/21576019816.json");
        let a = parse_activity_file(&path).expect("parse failed");

        assert_eq!(a.id.value(), 21576019816);
        assert_eq!(a.distance.meters(), 3841.18994140625);
        assert_eq!(a.duration.seconds(), 1749.72705078125);
        assert_eq!(a.avg_hr.map(|h| h.bpm()), Some(174));
        assert_eq!(a.max_hr.map(|h| h.bpm()), Some(188));
        assert_eq!(
            a.start_time.date(),
            NaiveDate::from_ymd_opt(2026, 1, 17).unwrap()
        );
    }

    #[test]
    fn parses_second_activity_correctly() {
        let path = activity_path("2025/02/18247389499.json");
        let a = parse_activity_file(&path).expect("parse failed");

        assert_eq!(a.id.value(), 18247389499);
        assert_eq!(a.distance.meters(), 1505.75);
        assert_eq!(a.avg_hr.map(|h| h.bpm()), Some(178));
        assert_eq!(a.max_hr.map(|h| h.bpm()), Some(192));
        assert_eq!(
            a.start_time.date(),
            NaiveDate::from_ymd_opt(2025, 2, 11).unwrap()
        );
    }

    #[test]
    fn splits_are_empty_and_terrain_is_unknown() {
        let path = activity_path("2026/01/21576019816.json");
        let a = parse_activity_file(&path).expect("parse failed");

        assert!(a.splits.is_empty());
        assert!(matches!(
            a.terrain,
            garmin_core::classification::Terrain::Unknown
        ));
    }

    #[test]
    fn loads_all_48_activities() {
        let manifest = env!("CARGO_MANIFEST_DIR");
        let base = Path::new(manifest).join("../../activity/activities");
        let activities = load_all_activities(&base).expect("load failed");

        assert_eq!(activities.len(), 48);
    }

    #[test]
    fn all_loaded_activities_have_nonzero_ids() {
        let manifest = env!("CARGO_MANIFEST_DIR");
        let base = Path::new(manifest).join("../../activity/activities");
        let activities = load_all_activities(&base).expect("load failed");

        for a in &activities {
            assert!(a.id.value() > 0);
        }
    }
}
