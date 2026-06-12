"""Offline sanity tests for the fetcher (no garminconnect required)."""

from pathlib import Path

import garmin_fetcher


def test_parse_args_defaults():
    args = garmin_fetcher.parse_args([])
    assert args.limit == 20
    assert args.days == 30
    assert args.raw_dir is None


def test_parse_args_overrides():
    args = garmin_fetcher.parse_args(
        ["--limit", "5", "--days", "7", "--raw-dir", "/tmp/raw"]
    )
    assert args.limit == 5
    assert args.days == 7
    assert args.raw_dir == Path("/tmp/raw")


def test_default_raw_dir_respects_env(monkeypatch):
    monkeypatch.setenv("GARMIN_LOG_DATA_DIR", "/custom/root")
    assert garmin_fetcher.default_raw_dir() == Path("/custom/root/raw")

    monkeypatch.delenv("GARMIN_LOG_DATA_DIR")
    monkeypatch.delenv("XDG_DATA_HOME", raising=False)
    assert garmin_fetcher.default_raw_dir() == (
        Path.home() / ".local" / "share" / "garmin-log-reporter" / "raw"
    )


def test_normalize_activity_matches_raw_contract():
    payload = {
        "activityId": 21576019816,
        "activityName": "Minillas Running",
        "activityType": {"typeKey": "running"},
        "startTimeLocal": "2026-01-17 08:00:10",
        "beginTimestamp": 1768651210000,
        "distance": 3841.18994140625,
        "duration": 1749.72705078125,
        "averageHR": 174.0,
        "maxHR": 188.0,
        "averageSpeed": 2.19,
        "maxSpeed": 3.36,
        "calories": 370.0,
        "steps": 4534,
        "locationName": "Minillas",
    }
    normalized = garmin_fetcher.normalize_activity(payload)
    # Field names must stay aligned with garmin_parser::RawActivity.
    assert set(normalized) == {
        "activity_id",
        "activity_name",
        "sport",
        "start_time",
        "timestamp",
        "distance_meters",
        "duration_seconds",
        "avg_hr",
        "max_hr",
        "avg_speed",
        "max_speed",
        "calories",
        "steps",
        "location",
    }
    assert normalized["activity_id"] == 21576019816
    assert normalized["sport"] == "running"
    assert normalized["start_time"] == "2026-01-17 08:00:10"


def test_missing_optional_fields_become_none():
    payload = {
        "activityId": 1,
        "activityName": None,
        "activityType": {"typeKey": "trail_running"},
        "startTimeLocal": "2026-01-01 06:00:00",
        "beginTimestamp": 1767247200000,
        "distance": 1000.0,
        "duration": 300.0,
    }
    normalized = garmin_fetcher.normalize_activity(payload)
    assert normalized["activity_name"] == ""
    assert normalized["avg_hr"] is None
    assert normalized["location"] is None


def test_is_running_filter():
    assert garmin_fetcher.is_running({"activityType": {"typeKey": "running"}})
    assert garmin_fetcher.is_running({"activityType": {"typeKey": "trail_running"}})
    assert not garmin_fetcher.is_running({"activityType": {"typeKey": "cycling"}})
    assert not garmin_fetcher.is_running({})
