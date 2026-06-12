mod format;
mod import;
mod sync;

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use garmin_core::activity::Activity;

use format::{Unit, format_distance, format_duration, format_pace, pace_seconds_per_km};

#[derive(Parser)]
#[command(name = "garmin-log", version, about = "Garmin running log reporter")]
struct Cli {
    /// Unit system for output
    #[arg(long, global = true, value_enum, default_value_t = Unit::Imperial)]
    unit: Unit,

    /// Override the data directory for this command only
    #[arg(long, global = true, value_name = "PATH")]
    data_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Import Garmin JSON activities from a file or directory
    Import { path: PathBuf },
    /// List stored activities
    List,
    /// Generate reports from stored activities
    Report {
        #[command(subcommand)]
        report: ReportCommand,
    },
    /// Show details for one stored activity
    Activity { id: u64 },
    /// Storage information
    Storage {
        #[command(subcommand)]
        storage: StorageCommand,
    },
    /// Fetch new activities from Garmin Connect and import them
    Sync {
        /// Maximum number of activities to fetch
        #[arg(long)]
        limit: Option<u32>,
        /// How many days back to fetch
        #[arg(long)]
        days: Option<u32>,
        /// Skip the Garmin fetch and only import existing raw files
        #[arg(long)]
        skip_fetch: bool,
        /// Override the raw download directory
        #[arg(long, value_name = "PATH")]
        raw_dir: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum ReportCommand {
    /// Overall totals across all stored activities
    Summary,
    /// Totals grouped by ISO week
    Weekly,
    /// Totals grouped by calendar month
    Monthly,
}

#[derive(Subcommand)]
enum StorageCommand {
    /// Print the active storage locations
    Path,
}

const EMPTY_STORAGE: &str =
    "No stored activities found. Import activities first:\n  garmin-log import <path>";

fn main() {
    // Die quietly instead of panicking when output is piped to e.g. `head`.
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let cli = Cli::parse();
    if let Err(message) = run(cli) {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let app_root = storage::resolve_app_root(cli.data_dir.as_deref()).map_err(|e| e.to_string())?;
    let data_root = storage::data_root_in(&app_root);
    let unit = cli.unit;

    match cli.command {
        Command::Import { path } => cmd_import(&path, &data_root),
        Command::List => cmd_list(&data_root, unit),
        Command::Report { report } => match report {
            ReportCommand::Summary => cmd_report_summary(&data_root, unit),
            ReportCommand::Weekly => cmd_report_weekly(&data_root, unit),
            ReportCommand::Monthly => cmd_report_monthly(&data_root, unit),
        },
        Command::Activity { id } => cmd_activity(&data_root, id, unit),
        Command::Storage {
            storage: StorageCommand::Path,
        } => cmd_storage_path(&app_root),
        Command::Sync {
            limit,
            days,
            skip_fetch,
            raw_dir,
        } => cmd_sync(&app_root, &data_root, limit, days, skip_fetch, raw_dir),
    }
}

fn load_stored(data_root: &Path) -> Result<Vec<Activity>, String> {
    let mut activities = storage::load_activities_in(data_root).map_err(|e| e.to_string())?;
    if activities.is_empty() {
        return Err(EMPTY_STORAGE.to_string());
    }
    activities.sort_by_key(|a| a.start_time);
    Ok(activities)
}

fn cmd_import(path: &Path, data_root: &Path) -> Result<(), String> {
    let files = import::collect_json_files(path)?;
    let stats = import::import_files(&files, data_root);
    print_import_stats(&stats, data_root);

    if stats.imported == 0 && stats.skipped == 0 && stats.failed > 0 {
        return Err("Import failed for every file.".to_string());
    }
    Ok(())
}

fn print_import_stats(stats: &import::ImportStats, data_root: &Path) {
    println!("Imported: {}", stats.imported);
    println!("Skipped existing: {}", stats.skipped);
    println!("Failed: {}", stats.failed);
    println!("Storage: {}", data_root.display());
    for (file, reason) in &stats.failures {
        eprintln!("  failed {}: {}", file.display(), reason);
    }
}

fn cmd_list(data_root: &Path, unit: Unit) -> Result<(), String> {
    let activities = load_stored(data_root)?;

    for a in &activities {
        let pace = pace_seconds_per_km(a.duration.seconds(), a.distance.meters())
            .map(|p| format_pace(p, unit))
            .unwrap_or_else(|| "-".to_string());
        let hr = a
            .avg_hr
            .map(|h| format!("{} bpm", h.bpm()))
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<12}  {}  {:>10}  {:>9}  {:>12}  {}",
            a.id.value(),
            a.start_time.date(),
            format_distance(a.distance.meters(), unit),
            format_duration(a.duration.seconds()),
            pace,
            hr,
        );
    }
    Ok(())
}

fn cmd_report_summary(data_root: &Path, unit: Unit) -> Result<(), String> {
    let activities = load_stored(data_root)?;
    let report = analytics::build_summary_report(&activities);

    println!("RUNNING SUMMARY");
    println!("---------------");
    println!("Total runs:     {}", report.total_activities);
    println!(
        "Total distance: {}",
        format_distance(report.total_distance_meters, unit)
    );
    println!(
        "Total time:     {}",
        format_duration(report.total_duration_seconds)
    );
    println!(
        "Average pace:   {}",
        report
            .average_pace_seconds_per_km
            .map(|p| format_pace(p, unit))
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "Average HR:     {}",
        report
            .average_heart_rate
            .map(|h| format!("{h} bpm"))
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "Max HR:         {}",
        report
            .max_heart_rate
            .map(|h| format!("{h} bpm"))
            .unwrap_or_else(|| "-".to_string())
    );
    if let (Some(id), Some(meters)) = (report.longest_activity_id, report.longest_distance_meters) {
        println!(
            "Longest run:    {} (activity {id})",
            format_distance(meters, unit)
        );
    }
    Ok(())
}

fn cmd_report_weekly(data_root: &Path, unit: Unit) -> Result<(), String> {
    let activities = load_stored(data_root)?;
    let weeks = analytics::build_weekly_reports(&activities);

    println!("WEEKLY REPORT");
    println!("-------------");
    for week in weeks {
        println!(
            "{}-W{:02}  runs: {:<3}  {:>10}  {:>9}  {:>12}  {}",
            week.iso_year,
            week.iso_week,
            week.activity_count,
            format_distance(week.total_distance_meters, unit),
            format_duration(week.total_duration_seconds),
            week.average_pace_seconds_per_km
                .map(|p| format_pace(p, unit))
                .unwrap_or_else(|| "-".to_string()),
            week.average_heart_rate
                .map(|h| format!("{h} bpm"))
                .unwrap_or_else(|| "-".to_string()),
        );
    }
    Ok(())
}

fn cmd_report_monthly(data_root: &Path, unit: Unit) -> Result<(), String> {
    let activities = load_stored(data_root)?;
    let months = analytics::build_monthly_reports(&activities);

    println!("MONTHLY REPORT");
    println!("--------------");
    for month in months {
        println!(
            "{}-{:02}  runs: {:<3}  {:>10}  {:>9}  {:>12}  {}",
            month.year,
            month.month,
            month.activity_count,
            format_distance(month.total_distance_meters, unit),
            format_duration(month.total_duration_seconds),
            month
                .average_pace_seconds_per_km
                .map(|p| format_pace(p, unit))
                .unwrap_or_else(|| "-".to_string()),
            month
                .average_heart_rate
                .map(|h| format!("{h} bpm"))
                .unwrap_or_else(|| "-".to_string()),
        );
    }
    Ok(())
}

fn cmd_activity(data_root: &Path, id: u64, unit: Unit) -> Result<(), String> {
    let activity = storage::load_activity_by_id_in(data_root, id)
        .map_err(|e| e.to_string())?
        .ok_or(format!("Activity not found: {id}"))?;

    println!("Activity {}", activity.id.value());
    println!("  Date:     {}", activity.start_time.value());
    println!("  Terrain:  {:?}", activity.terrain);
    println!(
        "  Distance: {}",
        format_distance(activity.distance.meters(), unit)
    );
    println!(
        "  Duration: {}",
        format_duration(activity.duration.seconds())
    );
    println!(
        "  Pace:     {}",
        pace_seconds_per_km(activity.duration.seconds(), activity.distance.meters())
            .map(|p| format_pace(p, unit))
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "  Avg HR:   {}",
        activity
            .avg_hr
            .map(|h| format!("{} bpm", h.bpm()))
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "  Max HR:   {}",
        activity
            .max_hr
            .map(|h| format!("{} bpm", h.bpm()))
            .unwrap_or_else(|| "-".to_string())
    );
    if let Some(device) = &activity.metadata.device {
        println!("  Device:   {device}");
    }
    if let Some(source) = &activity.metadata.source {
        println!("  Source:   {source}");
    }
    if let Some(notes) = &activity.metadata.notes {
        println!("  Notes:    {notes}");
    }
    println!("  Splits:   {}", activity.splits.len());
    Ok(())
}

fn cmd_storage_path(app_root: &Path) -> Result<(), String> {
    println!("Storage root: {}", app_root.display());
    println!(
        "Normalized:   {}",
        storage::data_root_in(app_root).display()
    );
    println!("Raw fetches:  {}", storage::raw_root_in(app_root).display());
    Ok(())
}

fn cmd_sync(
    app_root: &Path,
    data_root: &Path,
    limit: Option<u32>,
    days: Option<u32>,
    skip_fetch: bool,
    raw_dir: Option<PathBuf>,
) -> Result<(), String> {
    let raw_dir = raw_dir.unwrap_or_else(|| storage::raw_root_in(app_root));

    let fetch = if skip_fetch {
        None
    } else {
        Some(sync::run_fetcher(&raw_dir, limit, days)?)
    };

    if !raw_dir.exists() {
        return Err(format!(
            "No raw activities found in: {}\nRun a fetch first, or import local files:\n  garmin-log import <path>",
            raw_dir.display()
        ));
    }

    let files = import::collect_json_files(&raw_dir)?;
    let stats = import::import_files(&files, data_root);

    println!("GARMIN SYNC");
    println!("-----------");
    if let Some(fetch) = &fetch {
        println!("Fetched raw activities: {}", fetch.fetched);
        println!("New raw files: {}", fetch.new_files);
        println!("Existing raw files: {}", fetch.existing_files);
        if fetch.failed > 0 {
            println!("Fetch failures: {}", fetch.failed);
        }
    } else {
        println!("Fetch skipped (--skip-fetch)");
    }
    println!("Parsed activities: {}", stats.imported + stats.skipped);
    println!("Saved normalized activities: {}", stats.imported);
    println!("Skipped existing: {}", stats.skipped);
    println!("Failed: {}", stats.failed);
    for (file, reason) in &stats.failures {
        eprintln!("  failed {}: {}", file.display(), reason);
    }
    println!();
    println!("Storage:");
    println!("Raw: {}", raw_dir.display());
    println!("Normalized: {}", data_root.display());
    Ok(())
}
