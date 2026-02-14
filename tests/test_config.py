import os
import shutil
import datetime
from pathlib import Path

from garmin_client.config import Config


def test_env_loading(monkeypatch):
    monkeypatch.setenv("EMAIL", "test@mail.com")
    monkeypatch.setenv("PASSWORD", "secret")

    cfg = Config()

    assert cfg.email == "test@mail.com"
    assert cfg.password == "secret"


def test_defaults(monkeypatch):
    monkeypatch.delenv("EMAIL", raising=False)
    monkeypatch.delenv("PASSWORD", raising=False)

    cfg = Config()

    assert cfg.email is None
    assert cfg.password is None


def test_date_ranges():
    cfg = Config()

    today = datetime.date.today()

    assert cfg.today == today
    assert cfg.week_start == today - datetime.timedelta(days=7)
    assert cfg.month_start == today - datetime.timedelta(days=30)


def test_export_dir_created(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)

    cfg = Config()
    cfg.ensure_dirs()

    assert cfg.export_dir.exists()
    assert cfg.export_dir.is_dir()
