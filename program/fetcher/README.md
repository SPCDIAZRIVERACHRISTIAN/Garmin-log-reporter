# Garmin Fetcher

Python script that owns everything Garmin Connect: authentication (including
MFA), fetching recent running activities, and saving them as raw JSON files
that the Rust side parses. The Rust CLI (`garmin-log sync`) calls this script
as a subprocess.

## Setup

```bash
cd program/fetcher
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

## Credentials

```bash
export GARMIN_EMAIL="your-email"
export GARMIN_PASSWORD="your-password"
```

Credentials are read from the environment only — never stored on disk and
never printed. Session tokens are saved to `~/.garminconnect` (override with
`GARMINTOKENS`) so subsequent runs don't need to log in again.

## Usage

```bash
python garmin_fetcher.py --limit 20 --days 30
python garmin_fetcher.py --raw-dir /tmp/raw
```

Raw files go to `<GARMIN_LOG_DATA_DIR>/raw/{activity_id}.json`, defaulting to
`~/.local/share/garmin-log-reporter/raw/`.

## Output contract

- **stdout**: exactly one JSON summary object:

  ```json
  {
    "raw_dir": "/home/user/.local/share/garmin-log-reporter/raw",
    "fetched": 20,
    "new_files": 3,
    "existing_files": 17,
    "failed": 0,
    "files": ["..."]
  }
  ```

- **stderr**: human-readable progress and errors.

Exit codes: `0` ok, `2` missing credentials, `3` MFA required (non-interactive),
`4` auth/connection failure, `5` missing `garminconnect` dependency, `1` other.

## MFA limitation

If your Garmin account has MFA enabled, the first login must happen in an
interactive terminal so you can type the code. Run the fetcher directly once:

```bash
python garmin_fetcher.py --limit 1
```

After that the saved session token is reused and `garmin-log sync` works
non-interactively until the token expires.
