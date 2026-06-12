use std::path::Path;

use garmin_parser::loader::load_all_activities;

fn main() {
    let base = Path::new("../activity/activities");

    let activities = load_all_activities(base).expect("failed to load activities");

    println!("Loaded {} activities", activities.len());

    for a in &activities {
        println!("{:?}", a);
    }
}
