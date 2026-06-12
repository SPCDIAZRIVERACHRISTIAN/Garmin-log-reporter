//! Basic running analytics over `garmin_core` activities.
//!
//! All reports are pure functions of `&[Activity]` — no IO, no state.

use std::collections::BTreeMap;

use chrono::{Datelike, Utc};
use garmin_core::activity::Activity;

#[derive(Debug, Clone, PartialEq)]
pub struct SummaryReport {
    pub total_activities: usize,
    pub total_distance_meters: f64,
    pub total_duration_seconds: f64,
    pub average_pace_seconds_per_km: Option<f64>,
    pub average_heart_rate: Option<u32>,
    pub max_heart_rate: Option<u32>,
    pub longest_activity_id: Option<u64>,
    pub longest_distance_meters: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeeklyReport {
    pub iso_year: i32,
    pub iso_week: u32,
    pub activity_count: usize,
    pub total_distance_meters: f64,
    pub total_duration_seconds: f64,
    pub average_pace_seconds_per_km: Option<f64>,
    pub average_heart_rate: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonthlyReport {
    pub year: i32,
    pub month: u32,
    pub activity_count: usize,
    pub total_distance_meters: f64,
    pub total_duration_seconds: f64,
    pub average_pace_seconds_per_km: Option<f64>,
    pub average_heart_rate: Option<u32>,
}

pub fn build_summary_report(activities: &[Activity]) -> SummaryReport {
    let total_distance_meters: f64 = activities.iter().map(|a| a.distance.meters()).sum();
    let total_duration_seconds: f64 = activities.iter().map(|a| a.duration.seconds()).sum();

    let longest = activities.iter().max_by(|a, b| {
        a.distance
            .meters()
            .partial_cmp(&b.distance.meters())
            .expect("distance is never NaN")
    });

    SummaryReport {
        total_activities: activities.len(),
        total_distance_meters,
        total_duration_seconds,
        average_pace_seconds_per_km: average_pace(total_duration_seconds, total_distance_meters),
        average_heart_rate: average_heart_rate(activities),
        max_heart_rate: activities
            .iter()
            .filter_map(|a| a.max_hr.map(|h| h.bpm() as u32))
            .max(),
        longest_activity_id: longest.map(|a| a.id.value()),
        longest_distance_meters: longest.map(|a| a.distance.meters()),
    }
}

/// Weekly reports grouped by ISO week, sorted by year/week ascending.
pub fn build_weekly_reports(activities: &[Activity]) -> Vec<WeeklyReport> {
    let mut groups: BTreeMap<(i32, u32), Vec<&Activity>> = BTreeMap::new();
    for activity in activities {
        let week = activity.start_time.date().iso_week();
        groups
            .entry((week.year(), week.week()))
            .or_default()
            .push(activity);
    }

    groups
        .into_iter()
        .map(|((iso_year, iso_week), group)| weekly_report(iso_year, iso_week, &group))
        .collect()
}

/// Report for the current ISO week (UTC), or `None` if it has no activities.
pub fn build_current_week_report(activities: &[Activity]) -> Option<WeeklyReport> {
    let now = Utc::now().date_naive().iso_week();
    build_week_report(activities, now.year(), now.week())
}

/// Report for one specific ISO week, or `None` if it has no activities.
pub fn build_week_report(
    activities: &[Activity],
    iso_year: i32,
    iso_week: u32,
) -> Option<WeeklyReport> {
    let group: Vec<&Activity> = activities
        .iter()
        .filter(|a| {
            let week = a.start_time.date().iso_week();
            week.year() == iso_year && week.week() == iso_week
        })
        .collect();

    if group.is_empty() {
        return None;
    }
    Some(weekly_report(iso_year, iso_week, &group))
}

/// Monthly reports grouped by calendar month, sorted by year/month ascending.
pub fn build_monthly_reports(activities: &[Activity]) -> Vec<MonthlyReport> {
    let mut groups: BTreeMap<(i32, u32), Vec<&Activity>> = BTreeMap::new();
    for activity in activities {
        let date = activity.start_time.date();
        groups
            .entry((date.year(), date.month()))
            .or_default()
            .push(activity);
    }

    groups
        .into_iter()
        .map(|((year, month), group)| {
            let distance: f64 = group.iter().map(|a| a.distance.meters()).sum();
            let duration: f64 = group.iter().map(|a| a.duration.seconds()).sum();
            MonthlyReport {
                year,
                month,
                activity_count: group.len(),
                total_distance_meters: distance,
                total_duration_seconds: duration,
                average_pace_seconds_per_km: average_pace(duration, distance),
                average_heart_rate: average_heart_rate_refs(&group),
            }
        })
        .collect()
}

fn weekly_report(iso_year: i32, iso_week: u32, group: &[&Activity]) -> WeeklyReport {
    let distance: f64 = group.iter().map(|a| a.distance.meters()).sum();
    let duration: f64 = group.iter().map(|a| a.duration.seconds()).sum();
    WeeklyReport {
        iso_year,
        iso_week,
        activity_count: group.len(),
        total_distance_meters: distance,
        total_duration_seconds: duration,
        average_pace_seconds_per_km: average_pace(duration, distance),
        average_heart_rate: average_heart_rate_refs(group),
    }
}

/// Distance-weighted pace: total time over total kilometers.
fn average_pace(total_duration_seconds: f64, total_distance_meters: f64) -> Option<f64> {
    if total_distance_meters <= 0.0 {
        return None;
    }
    Some(total_duration_seconds / (total_distance_meters / 1000.0))
}

fn average_heart_rate(activities: &[Activity]) -> Option<u32> {
    mean_hr(activities.iter().filter_map(|a| a.avg_hr))
}

fn average_heart_rate_refs(activities: &[&Activity]) -> Option<u32> {
    mean_hr(activities.iter().filter_map(|a| a.avg_hr))
}

fn mean_hr(values: impl Iterator<Item = garmin_core::metrics::HeartRate>) -> Option<u32> {
    let bpms: Vec<u32> = values.map(|h| h.bpm() as u32).collect();
    if bpms.is_empty() {
        return None;
    }
    let sum: u32 = bpms.iter().sum();
    Some((sum as f64 / bpms.len() as f64).round() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use garmin_core::activity::{ActivityId, ActivityMetadata};
    use garmin_core::classification::Terrain;
    use garmin_core::metrics::HeartRate;
    use garmin_core::units::{Distance, Duration, Timestamp};

    fn activity(
        id: u64,
        ymd: (i32, u32, u32),
        meters: f64,
        seconds: f64,
        avg_hr: Option<u16>,
        max_hr: Option<u16>,
    ) -> Activity {
        let dt = Utc
            .with_ymd_and_hms(ymd.0, ymd.1, ymd.2, 6, 0, 0)
            .single()
            .unwrap();
        Activity::new(
            ActivityId::new(id),
            Terrain::Unknown,
            Timestamp::new(dt),
            Distance::from_meters(meters),
            Duration::from_seconds(seconds),
            avg_hr.map(HeartRate::new),
            max_hr.map(HeartRate::new),
            vec![],
            ActivityMetadata::empty(),
        )
    }

    #[test]
    fn empty_activity_list_produces_empty_summary() {
        let report = build_summary_report(&[]);
        assert_eq!(report.total_activities, 0);
        assert_eq!(report.total_distance_meters, 0.0);
        assert_eq!(report.total_duration_seconds, 0.0);
        assert!(report.average_pace_seconds_per_km.is_none());
        assert!(report.average_heart_rate.is_none());
        assert!(report.max_heart_rate.is_none());
        assert!(report.longest_activity_id.is_none());

        assert!(build_weekly_reports(&[]).is_empty());
        assert!(build_monthly_reports(&[]).is_empty());
        assert!(build_current_week_report(&[]).is_none());
    }

    #[test]
    fn summary_totals_are_summed() {
        let activities = vec![
            activity(1, (2026, 6, 1), 5000.0, 1500.0, Some(150), Some(170)),
            activity(2, (2026, 6, 2), 3000.0, 1000.0, Some(160), Some(180)),
        ];
        let report = build_summary_report(&activities);
        assert_eq!(report.total_activities, 2);
        assert_eq!(report.total_distance_meters, 8000.0);
        assert_eq!(report.total_duration_seconds, 2500.0);
    }

    #[test]
    fn average_pace_is_distance_weighted() {
        // 10 km in 3000 s => 300 s/km
        let activities = vec![
            activity(1, (2026, 6, 1), 4000.0, 1200.0, None, None),
            activity(2, (2026, 6, 2), 6000.0, 1800.0, None, None),
        ];
        let report = build_summary_report(&activities);
        let pace = report.average_pace_seconds_per_km.unwrap();
        assert!((pace - 300.0).abs() < 1e-9);
    }

    #[test]
    fn weekly_grouping_uses_iso_weeks() {
        // 2025-12-29 (Mon) and 2026-01-01 (Thu) are both ISO week 2026-W01.
        // 2026-01-05 (Mon) starts ISO week 2026-W02.
        let activities = vec![
            activity(1, (2025, 12, 29), 5000.0, 1500.0, None, None),
            activity(2, (2026, 1, 1), 5000.0, 1500.0, None, None),
            activity(3, (2026, 1, 5), 3000.0, 1000.0, None, None),
        ];
        let weeks = build_weekly_reports(&activities);
        assert_eq!(weeks.len(), 2);

        assert_eq!((weeks[0].iso_year, weeks[0].iso_week), (2026, 1));
        assert_eq!(weeks[0].activity_count, 2);
        assert_eq!(weeks[0].total_distance_meters, 10000.0);

        assert_eq!((weeks[1].iso_year, weeks[1].iso_week), (2026, 2));
        assert_eq!(weeks[1].activity_count, 1);
    }

    #[test]
    fn weekly_reports_are_sorted_ascending() {
        let activities = vec![
            activity(1, (2026, 3, 10), 5000.0, 1500.0, None, None),
            activity(2, (2025, 11, 3), 5000.0, 1500.0, None, None),
            activity(3, (2026, 1, 20), 5000.0, 1500.0, None, None),
        ];
        let weeks = build_weekly_reports(&activities);
        let keys: Vec<(i32, u32)> = weeks.iter().map(|w| (w.iso_year, w.iso_week)).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted);
    }

    #[test]
    fn longest_activity_is_detected() {
        let activities = vec![
            activity(1, (2026, 6, 1), 5000.0, 1500.0, None, None),
            activity(2, (2026, 6, 2), 12000.0, 4000.0, None, None),
            activity(3, (2026, 6, 3), 8000.0, 2500.0, None, None),
        ];
        let report = build_summary_report(&activities);
        assert_eq!(report.longest_activity_id, Some(2));
        assert_eq!(report.longest_distance_meters, Some(12000.0));
    }

    #[test]
    fn heart_rate_average_and_max() {
        let activities = vec![
            activity(1, (2026, 6, 1), 5000.0, 1500.0, Some(150), Some(170)),
            activity(2, (2026, 6, 2), 5000.0, 1500.0, Some(161), Some(190)),
            activity(3, (2026, 6, 3), 5000.0, 1500.0, None, None),
        ];
        let report = build_summary_report(&activities);
        // mean(150, 161) = 155.5 -> rounds to 156; activity without HR is ignored
        assert_eq!(report.average_heart_rate, Some(156));
        assert_eq!(report.max_heart_rate, Some(190));
    }

    #[test]
    fn specific_week_report_filters_correctly() {
        let activities = vec![
            activity(1, (2026, 6, 8), 5000.0, 1500.0, None, None),
            activity(2, (2026, 6, 1), 3000.0, 1000.0, None, None),
        ];
        // 2026-06-08 is a Monday => ISO week 2026-W24
        let report = build_week_report(&activities, 2026, 24).unwrap();
        assert_eq!(report.activity_count, 1);
        assert_eq!(report.total_distance_meters, 5000.0);

        assert!(build_week_report(&activities, 2026, 40).is_none());
    }

    #[test]
    fn monthly_grouping() {
        let activities = vec![
            activity(1, (2026, 5, 30), 5000.0, 1500.0, Some(150), None),
            activity(2, (2026, 6, 1), 3000.0, 1000.0, Some(160), None),
            activity(3, (2026, 6, 15), 7000.0, 2000.0, None, None),
        ];
        let months = build_monthly_reports(&activities);
        assert_eq!(months.len(), 2);
        assert_eq!((months[0].year, months[0].month), (2026, 5));
        assert_eq!((months[1].year, months[1].month), (2026, 6));
        assert_eq!(months[1].activity_count, 2);
        assert_eq!(months[1].total_distance_meters, 10000.0);
        assert_eq!(months[1].average_heart_rate, Some(160));
    }
}
