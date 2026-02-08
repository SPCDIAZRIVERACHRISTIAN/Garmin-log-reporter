# Garmin-log-reporter
A program to fetch your running data to feed it to ai so that it can coach you or give you suggestions on how to improve your run.
---
#Progress
---
##Phase 0
---
This was an exploratory phase where I found a Python library called garminconnect that does all the scrapping for me.
The plan is to leverage this library to fetch my running data from it call it from Rust and automize data fetching for it.
Then eliminate the need to use the lib for scrapping garmin and fetch it with Rust entirely.
---
###Features planned:
- **Rust wrapper to call the lib:** on this point rust will call my python class to fetch garmin gpx files and export them to json files that's it.

- **Program continuous running for fetching activities:** Make the program run continuosly fetch new activities and organize data by week.

- **Create a TUI to interact with:** Create a sort of informatic dashboard to see files and export to pc.

- **Create GUI for other normies XD:** Create a gui dashboard to display graphs information and files organized by weeks maybe add a DB to handle data before placing it in a json?
---
## Overview

This project extracts **running activity data** from Garmin Connect using a Python client (handles authentication + MFA) and processes the data in Rust for weekly organization and analysis.

The system is intentionally split by responsibility:

- **Python** → data extraction (auth, MFA, GPX export)
- **Rust** → data modeling, grouping, analysis, and CLI/TUI tooling

This separation keeps authentication complexity isolated and allows Rust to focus on correctness and structure.

---

## Architecture

Python
- Sign in to Garmin Connect
- Handle MFA challenges
- Fetch activities
- Filter running activities
- Download GPX files
- Parse GPX → normalized JSON (per run)

Rust
- Load run JSON files
- Deserialize into Rust structs
- Sort and group runs by ISO week
- Generate weekly JSON summaries
- Power future CLI / TUI / GUI

Python is treated as a **tool**, not a library.  
Rust consumes the JSON output as a stable data contract.

---

## Phases

Phase 0: Research & architecture definition ✅  
Phase 1: Python extractor (auth + MFA + GPX → JSON)  
Phase 2: Rust parser + weekly grouping  
Phase 3: CLI interface  
Phase 4: TUI / GUI (optional)

---

## Security Notes

- Credentials are **never stored on disk**
- Authentication uses Garmin’s normal flow, including **MFA**
- Passwords are provided via environment variables or runtime prompt
- Tokens/sessions live only in memory
- Exported JSON contains **no secrets**

If credentials are compromised during testing, rotate immediately.

---

## Data Contract (Draft)

Each running activity is exported as a single JSON file.

{
  "activity_id": 123,
  "type": "running",
  "start_time": "2026-02-01T05:30:00Z",
  "points": [
    {
      "t": "2026-02-01T05:30:05Z",
      "lat": 18.42,
      "lon": -66.11,
      "ele": 12.3
    }
  ]
}

Notes:
- Times are ISO-8601
- Elevation may be null or omitted
- Schema is intentionally minimal and stable
- Rust logic depends on this contract

---

## Design Principles

- Respect MFA — never bypass or hack auth
- Keep Python minimal and disposable
- Treat JSON as an API boundary
- Prefer boring, predictable data over cleverness
- Optimize later, only after correctness

---

## Future Work

- Expand run metrics (HR, cadence, pace)
- Add weekly and monthly summaries
- Introduce local database backend
- Replace Python extractor with native Rust (optional)

---

## Status

Phase 0 complete.  
Next step: implement Python extractor class.
