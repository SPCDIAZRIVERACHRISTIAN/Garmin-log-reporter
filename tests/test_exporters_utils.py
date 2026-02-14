import os
import json
import sys
import datetime

from garmin_client.exporters import DataExporter
from garmin_client.utils import safe_readkey, format_timedelta
from garmin_client.config import config


# =====================================================
# EXPORTER TESTS
# =====================================================


def test_save_json_pretty(tmp_path, monkeypatch):
    monkeypatch.setattr(config, "export_dir", tmp_path)

    data = {"a": 1, "b": 2}
    filepath = DataExporter.save_json(data, "testfile")

    file = tmp_path / "testfile.json"

    assert file.exists()
    assert filepath == str(file)

    content = json.loads(file.read_text())
    assert content == data


def test_save_json_compact(tmp_path, monkeypatch):
    monkeypatch.setattr(config, "export_dir", tmp_path)

    data = {"x": 10}
    DataExporter.save_json(data, "compact", pretty=False)

    file = tmp_path / "compact.json"
    text = file.read_text()

    # compact JSON shouldn't contain indentation spaces
    assert "\n    " not in text
    assert json.loads(text) == data


# =====================================================
# UTILS TESTS — format_timedelta
# =====================================================


def test_format_timedelta():
    td = datetime.timedelta(hours=1, minutes=2, seconds=3)
    result = format_timedelta(td)

    assert result == "1:02:03"


def test_format_timedelta_days():
    td = datetime.timedelta(days=1, seconds=5)
    result = format_timedelta(td)

    assert result == "24:00:05"


# =====================================================
# UTILS TESTS — safe_readkey
# =====================================================


def test_safe_readkey_tty(monkeypatch):
    monkeypatch.setattr(sys.stdin, "isatty", lambda: True)

    import readchar

    monkeypatch.setattr(readchar, "readkey", lambda: "a")

    result = safe_readkey()

    assert result == "a"


def test_safe_readkey_non_tty(monkeypatch):
    monkeypatch.setattr(sys.stdin, "isatty", lambda: False)
    monkeypatch.setattr("builtins.input", lambda _: "z")

    result = safe_readkey()

    assert result == "z"


def test_safe_readkey_exception(monkeypatch):
    monkeypatch.setattr(sys.stdin, "isatty", lambda: True)

    import readchar

    monkeypatch.setattr(readchar, "readkey", lambda: (_ for _ in ()).throw(Exception()))

    monkeypatch.setattr("builtins.input", lambda _: "x")

    result = safe_readkey()

    assert result == "x"
