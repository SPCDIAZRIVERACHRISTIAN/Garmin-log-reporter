import pytest
from garmin_client.client import GarminClient


class DummyConfig:
    email = "test@email.com"
    password = "pass123"


# ---------------------------
# Fixtures
# ---------------------------


@pytest.fixture
def client():
    return GarminClient(DummyConfig())


# ---------------------------
# Login Tests
# ---------------------------


def test_login_success(monkeypatch, client):

    def mock_safe_api_call(*args, **kwargs):
        return True, object(), None

    monkeypatch.setattr("garmin_client.client.safe_api_call", mock_safe_api_call)

    client.login()
    assert client.is_connected()


def test_login_failure(monkeypatch, client):

    def mock_safe_api_call(*args, **kwargs):
        return False, None, "Invalid credentials"

    monkeypatch.setattr("garmin_client.client.safe_api_call", mock_safe_api_call)

    with pytest.raises(Exception):
        client.login()


def test_login_mfa_required(monkeypatch, client):

    def mock_safe_api_call(*args, **kwargs):
        return False, None, "MFA required"

    monkeypatch.setattr("garmin_client.client.safe_api_call", mock_safe_api_call)

    with pytest.raises(Exception) as e:
        client.login()

    assert "MFA" in str(e.value)


# ---------------------------
# Connection State
# ---------------------------


def test_is_connected_false(client):
    assert not client.is_connected()


def test_is_connected_true(client):
    client.api = object()
    assert client.is_connected()


# ---------------------------
# _call behavior
# ---------------------------


def test_call_success(monkeypatch, client):

    def mock_safe_api_call(*args, **kwargs):
        return True, {"data": 123}, None

    monkeypatch.setattr("garmin_client.client.safe_api_call", mock_safe_api_call)

    result = client._call(lambda: None)
    assert result == {"data": 123}


def test_call_failure(monkeypatch, client):

    def mock_safe_api_call(*args, **kwargs):
        return False, None, "API down"

    monkeypatch.setattr("garmin_client.client.safe_api_call", mock_safe_api_call)

    with pytest.raises(Exception):
        client._call(lambda: None)


# ---------------------------
# Activity Fetching
# ---------------------------


def test_get_activities_not_connected(client):
    with pytest.raises(RuntimeError):
        client.get_activities("2024-01-01", "2024-01-07")


def test_get_activity_not_connected(client):
    with pytest.raises(RuntimeError):
        client.get_activity(123)


def test_get_activities_success(monkeypatch, client):

    class FakeAPI:
        def get_activities_by_date(self, start, end):
            return [{"id": 1}]

    client.api = FakeAPI()

    def mock_safe_api_call(method, *args, **kwargs):
        return True, method(*args), None

    monkeypatch.setattr("garmin_client.client.safe_api_call", mock_safe_api_call)

    result = client.get_activities("2024", "2025")
    assert result == [{"id": 1}]


def test_get_activity_success(monkeypatch, client):

    class FakeAPI:
        def get_activity_details(self, activity_id):
            return {"id": activity_id}

    client.api = FakeAPI()

    def mock_safe_api_call(method, *args, **kwargs):
        return True, method(*args), None

    monkeypatch.setattr("garmin_client.client.safe_api_call", mock_safe_api_call)

    result = client.get_activity(42)
    assert result["id"] == 42


# ---------------------------
# Edge Case — Garmin returns garbage
# ---------------------------


def test_api_returns_none(monkeypatch, client):

    class FakeAPI:
        def get_activities_by_date(self, a, b):
            return None

    client.api = FakeAPI()

    def mock_safe_api_call(method, *args, **kwargs):
        return True, method(*args), None

    monkeypatch.setattr("garmin_client.client.safe_api_call", mock_safe_api_call)

    result = client.get_activities("a", "b")
    assert result is None


# ---------------------------
# Edge Case — Dev passes wrong type
# ---------------------------


def test_invalid_activity_id(monkeypatch, client):

    class FakeAPI:
        def get_activity_details(self, activity_id):
            raise TypeError("invalid id")

    client.api = FakeAPI()

    def mock_safe_api_call(method, *args, **kwargs):
        return False, None, "invalid id"

    monkeypatch.setattr("garmin_client.client.safe_api_call", mock_safe_api_call)

    with pytest.raises(Exception):
        client.get_activity("not-an-int")
