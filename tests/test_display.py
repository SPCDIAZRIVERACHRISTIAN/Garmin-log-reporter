import json
from pathlib import Path

from garmin_client.display import call_and_display, _display_single, _display_group
from garmin_client.config import config


# -------------------------
# Helpers
# -------------------------


def fake_success():
    return {"hello": "world"}


def fake_none():
    return None


def fake_error():
    raise ValueError("boom")


# -------------------------
# Single display tests
# -------------------------


def test_display_success(tmp_path, monkeypatch, capsys):
    monkeypatch.setattr(config, "export_dir", tmp_path)

    success, result = call_and_display(fake_success)

    captured = capsys.readouterr()

    assert success is True
    assert result == {"hello": "world"}
    assert "API Call" in captured.out

    # file written
    file = tmp_path / "response.json"
    assert file.exists()


def test_display_none(tmp_path, monkeypatch, capsys):
    monkeypatch.setattr(config, "export_dir", tmp_path)

    success, result = call_and_display(fake_none)

    captured = capsys.readouterr()

    assert success is True
    assert result is None
    assert "No data returned" in captured.out


def test_display_error(tmp_path, monkeypatch, capsys):
    monkeypatch.setattr(config, "export_dir", tmp_path)

    success, result = call_and_display(fake_error)

    captured = capsys.readouterr()

    assert success is False
    assert result is None
    assert "ERROR" in captured.out


# -------------------------
# Direct display function tests
# -------------------------


def test_display_single_direct(tmp_path, monkeypatch):
    monkeypatch.setattr(config, "export_dir", tmp_path)

    _display_single("test_call", {"a": 1})

    file = tmp_path / "response.json"
    assert file.exists()

    content = file.read_text()
    assert "test_call" in content
    assert '"a": 1' in content


# -------------------------
# Group display tests
# -------------------------


def test_display_group(tmp_path, monkeypatch, capsys):
    monkeypatch.setattr(config, "export_dir", tmp_path)

    responses = [
        ("call1", {"x": 1}),
        ("call2", {"y": 2}),
    ]

    _display_group("GroupTest", responses)

    captured = capsys.readouterr()

    assert "GroupTest" in captured.out

    file = tmp_path / "response.json"
    assert file.exists()

    text = file.read_text()
    assert "call1" in text
    assert "call2" in text
