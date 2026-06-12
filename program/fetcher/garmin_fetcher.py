#!/usr/bin/env python3
"""Fetch running activities from Garmin Connect and save them as raw JSON files.

This script owns everything Garmin Connect: authentication, MFA, fetching,
and normalizing API payloads into the raw JSON contract that the Rust side
(`garmin_parser::RawActivity`) consumes. Rust calls it as a subprocess.

Output contract:
  stdout  - exactly one machine-readable JSON summary object
  stderr  - human-readable progress and error messages

Exit codes:
  0  success
  1  unexpected error
  2  missing credentials
  3  MFA required but not available non-interactively
  4  authentication or connection failure
  5  the garminconnect dependency is not installed
"""

import argparse
import datetime
import json
import os
import sys
from pathlib import Path

EXIT_OK = 0
EXIT_UNEXPECTED = 1
EXIT_NO_CREDENTIALS = 2
EXIT_MFA_REQUIRED = 3
EXIT_AUTH_FAILED = 4
EXIT_MISSING_DEPENDENCY = 5


def eprint(*args):
    print(*args, file=sys.stderr)


def default_raw_dir() -> Path:
    """<GARMIN_LOG_DATA_DIR or ~/.local/share/garmin-log-reporter>/raw"""
    root = os.environ.get("GARMIN_LOG_DATA_DIR")
    if root:
        return Path(root) / "raw"
    xdg = os.environ.get("XDG_DATA_HOME")
    base = Path(xdg) if xdg else Path.home() / ".local" / "share"
    return base / "garmin-log-reporter" / "raw"


def normalize_activity(act: dict) -> dict:
    """Map a Garmin Connect activity payload to the raw JSON contract.

    Must stay field-compatible with `garmin_parser::RawActivity` on the Rust
    side (and with the historical garmin_client.exporters mapping).
    """
    return {
        "activity_id": act["activityId"],
        "activity_name": act.get("activityName") or "",
        "sport": act["activityType"]["typeKey"],
        "start_time": act["startTimeLocal"],
        "timestamp": act["beginTimestamp"],
        "distance_meters": act["distance"],
        "duration_seconds": act["duration"],
        "avg_hr": act.get("averageHR"),
        "max_hr": act.get("maxHR"),
        "avg_speed": act.get("averageSpeed"),
        "max_speed": act.get("maxSpeed"),
        "calories": act.get("calories"),
        "steps": act.get("steps"),
        "location": act.get("locationName"),
    }


def is_running(act: dict) -> bool:
    type_key = (act.get("activityType") or {}).get("typeKey", "")
    return "running" in type_key


def parse_args(argv=None):
    parser = argparse.ArgumentParser(
        description="Fetch running activities from Garmin Connect as raw JSON."
    )
    parser.add_argument(
        "--limit", type=int, default=20, help="max activities to keep (default: 20)"
    )
    parser.add_argument(
        "--days", type=int, default=30, help="days back to fetch (default: 30)"
    )
    parser.add_argument(
        "--raw-dir",
        type=Path,
        default=None,
        help="where to save raw JSON files (default: <data root>/raw)",
    )
    return parser.parse_args(argv)


def login(email: str, password: str):
    """Authenticate against Garmin Connect, reusing saved tokens when possible.

    Returns an authenticated Garmin client or exits via SystemExit with one of
    the documented exit codes.
    """
    from garminconnect import Garmin

    tokenstore = os.environ.get("GARMINTOKENS", "~/.garminconnect")

    # 1. Saved session. Validate with a cheap call so stale tokens fall
    #    through to a fresh credential login.
    try:
        api = Garmin()
        api.garth.load(tokenstore)
        api.get_full_name()
        eprint(f"Using saved Garmin session from {tokenstore}")
        return api
    except Exception:
        pass

    # 2. Credential login (may require MFA).
    try:
        api = Garmin(email, password, return_on_mfa=True)
        result, state = api.login()
        if result == "needs_mfa":
            if sys.stdin.isatty():
                code = input("Enter Garmin MFA code: ").strip()
                api.resume_login(state, code)
            else:
                eprint(
                    "Garmin requires an MFA code, which needs an interactive "
                    "terminal.\nRun the fetcher directly once to complete MFA "
                    "and save a session token:\n"
                    "  python program/fetcher/garmin_fetcher.py --limit 1"
                )
                raise SystemExit(EXIT_MFA_REQUIRED)
        api.garth.dump(tokenstore)
        eprint(f"Logged in to Garmin Connect; session saved to {tokenstore}")
        return api
    except SystemExit:
        raise
    except Exception as e:
        eprint(f"Garmin authentication failed: {e}")
        eprint(f"If this persists, delete the token store ({tokenstore}) and retry.")
        raise SystemExit(EXIT_AUTH_FAILED)


def main(argv=None) -> int:
    args = parse_args(argv)

    email = os.environ.get("GARMIN_EMAIL")
    password = os.environ.get("GARMIN_PASSWORD")
    if not email or not password:
        eprint(
            "Garmin credentials are missing.\n"
            "Set:\n"
            "  GARMIN_EMAIL=<email>\n"
            "  GARMIN_PASSWORD=<password>"
        )
        return EXIT_NO_CREDENTIALS

    try:
        import garminconnect  # noqa: F401
    except ImportError:
        eprint(
            "The 'garminconnect' package is not installed.\n"
            "Install the fetcher dependencies:\n"
            "  cd program/fetcher\n"
            "  python -m venv .venv\n"
            "  source .venv/bin/activate\n"
            "  pip install -r requirements.txt"
        )
        return EXIT_MISSING_DEPENDENCY

    api = login(email, password)

    end = datetime.date.today()
    start = end - datetime.timedelta(days=args.days)
    eprint(f"Fetching activities from {start} to {end}...")
    try:
        activities = api.get_activities_by_date(start.isoformat(), end.isoformat())
    except Exception as e:
        eprint(f"Failed to fetch activities from Garmin Connect: {e}")
        return EXIT_AUTH_FAILED

    running = [a for a in (activities or []) if is_running(a)]
    if args.limit and args.limit > 0:
        running = running[: args.limit]
    eprint(f"Found {len(running)} running activities")

    raw_dir = args.raw_dir if args.raw_dir else default_raw_dir()
    raw_dir.mkdir(parents=True, exist_ok=True)

    new_files = 0
    existing_files = 0
    failed = 0
    files = []
    for act in running:
        try:
            normalized = normalize_activity(act)
            file_path = raw_dir / f"{normalized['activity_id']}.json"
            if file_path.exists():
                existing_files += 1
            else:
                file_path.write_text(
                    json.dumps(normalized, indent=4, ensure_ascii=False),
                    encoding="utf-8",
                )
                new_files += 1
            files.append(str(file_path))
        except Exception as e:
            failed += 1
            eprint(f"Failed to save activity {act.get('activityId', '?')}: {e}")

    summary = {
        "raw_dir": str(raw_dir),
        "fetched": len(running),
        "new_files": new_files,
        "existing_files": existing_files,
        "failed": failed,
        "files": files,
    }
    print(json.dumps(summary))
    return EXIT_OK


if __name__ == "__main__":
    try:
        sys.exit(main())
    except SystemExit:
        raise
    except KeyboardInterrupt:
        eprint("Interrupted.")
        sys.exit(EXIT_UNEXPECTED)
    except Exception as e:
        eprint(f"Unexpected error: {e}")
        sys.exit(EXIT_UNEXPECTED)
