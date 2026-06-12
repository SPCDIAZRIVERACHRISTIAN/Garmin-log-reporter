use clap::ValueEnum;

const METERS_PER_MILE: f64 = 1609.344;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Unit {
    Metric,
    Imperial,
}

impl Unit {
    pub fn distance_label(self) -> &'static str {
        match self {
            Unit::Metric => "km",
            Unit::Imperial => "mi",
        }
    }

    pub fn pace_label(self) -> &'static str {
        match self {
            Unit::Metric => "min/km",
            Unit::Imperial => "min/mi",
        }
    }
}

/// Format meters as "12.34 km" or "7.67 mi".
pub fn format_distance(meters: f64, unit: Unit) -> String {
    let value = match unit {
        Unit::Metric => meters / 1000.0,
        Unit::Imperial => meters / METERS_PER_MILE,
    };
    format!("{:.2} {}", value, unit.distance_label())
}

/// Format a metric pace (seconds per km) as "5:12 min/km" or "8:22 min/mi".
pub fn format_pace(seconds_per_km: f64, unit: Unit) -> String {
    let seconds_per_unit = match unit {
        Unit::Metric => seconds_per_km,
        Unit::Imperial => seconds_per_km * METERS_PER_MILE / 1000.0,
    };
    let total = seconds_per_unit.round() as u64;
    format!("{}:{:02} {}", total / 60, total % 60, unit.pace_label())
}

/// Format total seconds as "h:mm:ss".
pub fn format_duration(seconds: f64) -> String {
    let total = seconds.round() as u64;
    format!(
        "{}:{:02}:{:02}",
        total / 3600,
        (total % 3600) / 60,
        total % 60
    )
}

/// Pace of one activity in seconds per km, or `None` for zero-distance activities.
pub fn pace_seconds_per_km(duration_seconds: f64, distance_meters: f64) -> Option<f64> {
    if distance_meters <= 0.0 {
        return None;
    }
    Some(duration_seconds / (distance_meters / 1000.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_formatting() {
        assert_eq!(format_distance(5000.0, Unit::Metric), "5.00 km");
        assert_eq!(format_distance(1609.344, Unit::Imperial), "1.00 mi");
    }

    #[test]
    fn pace_formatting() {
        // 300 s/km = 5:00 min/km = ~8:03 min/mi
        assert_eq!(format_pace(300.0, Unit::Metric), "5:00 min/km");
        assert_eq!(format_pace(300.0, Unit::Imperial), "8:03 min/mi");
    }

    #[test]
    fn duration_formatting() {
        assert_eq!(format_duration(59.4), "0:00:59");
        assert_eq!(format_duration(3675.0), "1:01:15");
    }

    #[test]
    fn zero_distance_has_no_pace() {
        assert!(pace_seconds_per_km(100.0, 0.0).is_none());
        assert!(pace_seconds_per_km(300.0, 1000.0).unwrap() == 300.0);
    }
}
