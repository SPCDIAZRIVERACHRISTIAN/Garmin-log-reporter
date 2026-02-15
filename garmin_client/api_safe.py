from garth.exc import GarthHTTPError

from garminconnect import (
    GarminConnectAuthenticationError,
    GarminConnectConnectionError,
)


def safe_api_call(api_method, *args, method_name: str | None = None, **kwargs):
    """Centralized API call wrapper with comprehensive error handling.

    This function provides unified error handling for all Garmin Connect API calls.
    It handles common HTTP errors (400, 401, 403, 404, 429, 500, 503) with
    user-friendly messages and provides consistent error reporting.

    Usage:
        success, result, error_msg = safe_api_call(api.get_user_summary)

    Args:
        api_method: The API method to call
        *args: Positional arguments for the API method
        method_name: Human-readable name for the API method (optional)
        **kwargs: Keyword arguments for the API method

    Returns:
        tuple: (success: bool, result: Any, error_message: str|None)

    """
    if method_name is None:
        method_name = getattr(api_method, "__name__", str(api_method))

    try:
        result = api_method(*args, **kwargs)
        return True, result, None

    except GarthHTTPError as e:
        # Handle specific HTTP errors more gracefully
        error_str = str(e)

        # Extract status code more reliably
        status_code = None
        if hasattr(e, "response") and hasattr(e.response, "status_code"):
            status_code = e.response.status_code

        # Handle specific status codes
        if status_code == 400 or ("400" in error_str and "Bad Request" in error_str):
            error_msg = "Endpoint not available (400 Bad Request) - This feature may not be enabled for your account or region"
            # Don't print for 400 errors as they're often expected for unavailable features
        elif status_code == 401 or "401" in error_str:
            error_msg = (
                "Authentication required (401 Unauthorized) - Please re-authenticate"
            )
            print(f"⚠️ {method_name} failed: {error_msg}")
        elif status_code == 403 or "403" in error_str:
            error_msg = "Access denied (403 Forbidden) - Your account may not have permission for this feature"
            print(f"⚠️ {method_name} failed: {error_msg}")
        elif status_code == 404 or "404" in error_str:
            error_msg = (
                "Endpoint not found (404) - This feature may have been moved or removed"
            )
            print(f"⚠️ {method_name} failed: {error_msg}")
        elif status_code == 429 or "429" in error_str:
            error_msg = (
                "Rate limit exceeded (429) - Please wait before making more requests"
            )
            print(f"⚠️ {method_name} failed: {error_msg}")
        elif status_code == 500 or "500" in error_str:
            error_msg = "Server error (500) - Garmin's servers are experiencing issues"
            print(f"⚠️ {method_name} failed: {error_msg}")
        elif status_code == 503 or "503" in error_str:
            error_msg = "Service unavailable (503) - Garmin's servers are temporarily unavailable"
            print(f"⚠️ {method_name} failed: {error_msg}")
        else:
            error_msg = f"HTTP error: {e}"

        print(f"⚠️ {method_name} failed: {error_msg}")
        return False, None, error_msg

    except GarminConnectAuthenticationError as e:
        error_msg = f"Authentication issue: {e}"
        print(f"⚠️ {method_name} failed: {error_msg}")
        return False, None, error_msg

    except GarminConnectConnectionError as e:
        error_msg = f"Connection issue: {e}"
        print(f"⚠️ {method_name} failed: {error_msg}")
        return False, None, error_msg

    except Exception as e:
        error_msg = f"Unexpected error: {e}"
        print(f"⚠️ {method_name} failed: {error_msg}")
        return False, None, error_msg


def safe_call_for_group(
    api_method,
    *args,
    method_name: str | None = None,
    api_call_desc: str | None = None,
    **kwargs,
):
    """Safe API call wrapper that returns result suitable for grouped display.

    Args:
        api_method: The API method to call
        *args: Positional arguments for the API method
        method_name: Human-readable name for the API method (optional)
        api_call_desc: Description for display purposes (optional)
        **kwargs: Keyword arguments for the API method

    Returns:
        tuple: (api_call_description: str, result: Any) - suitable for grouped display

    """
    if method_name is None:
        method_name = getattr(api_method, "__name__", str(api_method))

    if api_call_desc is None:
        # Try to construct a reasonable description
        args_str = ", ".join(str(arg) for arg in args)
        kwargs_str = ", ".join(f"{k}={v}" for k, v in kwargs.items())
        all_args = ", ".join(filter(None, [args_str, kwargs_str]))
        api_call_desc = f"{method_name}({all_args})"

    success, result, error_msg = safe_api_call(
        api_method, *args, method_name=method_name, **kwargs
    )

    if success:
        return api_call_desc, result
    return f"{api_call_desc} [ERROR]", {"error": error_msg}
