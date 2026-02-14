import pytest

from garmin_client.api_safe import safe_api_call, safe_call_for_group


# -------------------------
# Helpers
# -------------------------


def success_func():
    return {"status": "ok"}


def arg_func(x, y):
    return x + y


def raise_auth():
    from garminconnect import GarminConnectAuthenticationError

    raise GarminConnectAuthenticationError("bad auth")


def raise_conn():
    from garminconnect import GarminConnectConnectionError

    raise GarminConnectConnectionError("network down")


def raise_generic():
    raise ValueError("boom")


# -------------------------
# safe_api_call tests
# -------------------------


def test_safe_api_success():
    success, result, err = safe_api_call(success_func)

    assert success is True
    assert result == {"status": "ok"}
    assert err is None


def test_safe_api_with_args():
    success, result, err = safe_api_call(arg_func, 2, 3)

    assert success is True
    assert result == 5
    assert err is None


def test_auth_error():
    success, result, err = safe_api_call(raise_auth)

    assert success is False
    assert result is None
    assert "Authentication issue" in err


def test_connection_error():
    success, result, err = safe_api_call(raise_conn)

    assert success is False
    assert result is None
    assert "Connection issue" in err


def test_generic_exception():
    success, result, err = safe_api_call(raise_generic)

    assert success is False
    assert result is None
    assert "Unexpected error" in err


# -------------------------
# safe_call_for_group tests
# -------------------------


def test_group_success():
    desc, result = safe_call_for_group(success_func)

    assert desc.startswith("success_func")
    assert result == {"status": "ok"}


def test_group_error():
    desc, result = safe_call_for_group(raise_generic)

    assert "[ERROR]" in desc
    assert isinstance(result, dict)
    assert "error" in result
