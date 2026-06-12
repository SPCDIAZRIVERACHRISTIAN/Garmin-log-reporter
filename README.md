# Garmin Log Reporter

A CLI tool that syncs your running activities from Garmin Connect, stores them
as normalized JSON on your machine, and generates running reports (summary,
weekly, monthly) from the terminal.

**Design split:** Python owns Garmin Connect (auth, MFA, fetching). Rust owns
the app (parsing, storage, analytics, CLI). They meet at a stable raw-JSON
contract on disk.

---

## MVP flow

```bash
garmin-log sync              # fetch from Garmin Connect + import
garmin-log report summary
garmin-log report weekly
```

Local import stays available for testing, offline use, and previously
downloaded data:

```bash
garmin-log import ./activity/activities
```

---

## Workspace overview

```text
program/
├── garmin_core/    # Domain model: Activity, Distance, Duration, HeartRate, ...
│                   # Stable foundation — depends on nothing else in the workspace.
├── garmin_parser/  # Raw Garmin JSON -> typed Activity
├── storage/        # Save/load normalized activities on disk
├── analytics/      # Summary / weekly / monthly reports from activities
├── garmin_cli/     # The `garmin-log` binary (clap)
├── fetcher/        # Python: Garmin Connect auth + fetch (called by `sync`)
├── gui/            # Stub (post-MVP)
└── tui/            # Stub (post-MVP)
```

Architecture rule: dependencies flow inward toward `garmin_core`, never
outward from it. `garmin_core` knows nothing about parsing, storage, or UIs.

---

## Build & test

Requires Rust (edition 2024 toolchain) and Python 3 (for sync only).

```bash
cd program
cargo build --all
cargo test --all
```

Python fetcher tests (optional, needs pytest):

```bash
program/fetcher/.venv/bin/python -m pytest program/fetcher/test_garmin_fetcher.py
```

---

## Python fetcher setup (required for `sync`)

```bash
cd program/fetcher
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

Credentials and configuration:

```bash
export GARMIN_EMAIL="your-email"
export GARMIN_PASSWORD="your-password"
export GARMIN_LOG_DATA_DIR="$HOME/.local/share/garmin-log-reporter"   # optional
```

Credentials are never stored on disk or printed. Garmin session tokens are
saved to `~/.garminconnect` (override with `GARMINTOKENS`) so repeat syncs
don't re-authenticate.

---

## Usage

```bash
cd program

# Sync from Garmin Connect (the main flow)
cargo run -p garmin_cli -- sync --limit 20
cargo run -p garmin_cli -- report summary
cargo run -p garmin_cli -- report weekly

# Fallback: import local raw JSON files (file or directory)
cargo run -p garmin_cli -- import ../activity/activities

# Other commands
cargo run -p garmin_cli -- list
cargo run -p garmin_cli -- report monthly
cargo run -p garmin_cli -- activity 21576019816
cargo run -p garmin_cli -- storage path
```

`sync` flags: `--limit N`, `--days N`, `--skip-fetch` (re-import existing raw
files without calling Garmin), `--raw-dir <path>`.

Global flags (every command):

- `--unit metric|imperial` — output units (default: imperial → miles, min/mi;
  metric → km, min/km)
- `--data-dir <path>` — override the storage root for this command only

### Example report

```text
RUNNING SUMMARY
---------------
Total runs:     48
Total distance: 125.45 mi
Total time:     26:04:51
Average pace:   12:28 min/mi
Average HR:     169 bpm
Max HR:         197 bpm
Longest run:    7.00 mi (activity 18473439990)
```

---

## Storage layout

Everything lives under one root, resolved in this order: `--data-dir` flag →
`GARMIN_LOG_DATA_DIR` env var → `~/.local/share/garmin-log-reporter`.

```text
garmin-log-reporter/
├── raw/                       # untouched fetcher output
│   └── 21576019816.json
└── data/                      # normalized activities
    └── 2026/
        └── 01/
            └── 21576019816.json
```

Existing activities are never overwritten — re-imports and re-syncs skip them.

---

## Limitations (MVP)

- CLI only — no TUI, no GUI, no charts yet.
- Garmin MFA/2FA needs one interactive login: run
  `python program/fetcher/garmin_fetcher.py --limit 1` once to enter the code
  and save a session token; `sync` works non-interactively after that.
- Sync depends on Garmin Connect's external behavior (rate limits, API
  changes).
- Local import works fully offline and independently of sync.
- Raw activities don't include splits or terrain yet, so stored activities
  have empty splits and `Unknown` terrain.
- No database — plain JSON files on disk.

---

## Phase history

- **Phase 0** — research; chose the `garminconnect` Python library for
  fetching, Rust for everything else.
- **Phase 1** — `garmin_core` domain model (typed units, metrics, errors).
- **Phase 2** — `garmin_parser`: raw Garmin JSON → typed `Activity`, tested
  against 48 real activities.
- **Phase 3 (MVP)** — storage, analytics, `garmin-log` CLI, and Garmin Connect
  sync via the Python fetcher.
