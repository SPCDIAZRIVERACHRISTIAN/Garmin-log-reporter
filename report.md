# Garmin Log Reporter — Project Status Report

## What it is
A Rust workspace with 6 crates meant to parse, analyze, and visualize Garmin running activity data stored as JSON files.

---

## Phase history

### Phase 1 — Make garmin_parser compile (done)

**Problem:** The `RawActivity → Activity` conversion didn't compile. The raw struct had flat primitives (`distance_meters: f64`, `avg_hr: Option<f64>`, etc.) but `Activity` expected typed wrappers (`Distance`, `HeartRate`, `Timestamp`). ~14 type/field mismatch errors.

**Fix:**
- Rewrote `From<RawActivity> for Activity` in `garmin_parser` field by field:
  - `u64 → ActivityId::new(...)`
  - `String` — parsed as `"%Y-%m-%d %H:%M:%S"` in the parser, handed to `Timestamp::new(DateTime<Utc>)`
  - `f64 → Distance::from_meters(...)` / `Duration::from_seconds(...)`
  - `Option<f64> → Option<HeartRate>` via `.map(|v| HeartRate::new(v as u16))`
  - `splits`, `terrain`, `metadata` filled with empty/unknown defaults (no raw data for them yet)
- `cargo check` passes clean.

**Correction (after Phase 2):** An initial attempt added `Timestamp::from_garmin_str` to `garmin_core`, leaking Garmin's date format into the domain model. Removed. Core only exposes `Timestamp::new(DateTime<Utc>)`. The parser owns the format-specific parsing — core does not bend to parser.

### Phase 2 — Parser tests with real activity JSON (done)

**5 tests in `garmin_parser/src/parser.rs`**, all passing, using real files from `activity/activities/`:

| Test | What it checks |
|------|---------------|
| `parses_all_fields_correctly` | id, distance, duration, avg_hr, max_hr, date on `21576019816.json` |
| `parses_second_activity_correctly` | Same spot-checks on `18247389499.json` |
| `splits_are_empty_and_terrain_is_unknown` | Phase 1 defaults are in place |
| `loads_all_48_activities` | Bulk load via `load_all_activities` returns exactly 48 |
| `all_loaded_activities_have_nonzero_ids` | Every activity in the full set has a valid id |

---

## Current state

**`garmin_core`** — Type system solid and compiling:
- Strongly-typed wrappers: `Distance`, `Duration`, `Pace`, `HeartRate`, `Cadence`, `Elevation`, `Calories`, `TrainingEffect`
- `Activity` struct with splits, metadata, timestamps
- Intensity classification based on HR zones (Recovery → VO2Max)
- Custom error types (`ParseError`, `ValidationError`, `ClassError`)

**`garmin_parser`** — Fully working:
- Walks a directory tree for `.json` files
- Deserializes `RawActivity` via serde and converts to typed `Activity`
- 5 passing tests against 48 real Garmin JSON files

**Known gaps (deferred):**
- Terrain classification always returns `Unknown` — no raw field maps to it yet
- `splits` always empty — raw JSON has no split data
- `ActivityMetadata` always empty — `location`/`activity_name` from raw not yet mapped

---

## What's still stubbed (empty `Hello, world!`)

| Crate | Intent |
|-------|--------|
| `analytics` | Reporting and stats |
| `storage` | Persistence layer |
| `gui` | Graphical UI |
| `tui` | Terminal UI |

---

## Planned phases

| Phase | Goal | Status |
|-------|------|--------|
| 1 | Make `garmin_parser` compile | done |
| 2 | Parser tests with real activity JSON | done |
| 3 | Basic storage: save/load `Vec<Activity>` | next |
| 4 | Analytics: weekly mileage, pace, HR summary | — |
| 5 | CLI command to run reports | — |
| 6 | TUI | — |
