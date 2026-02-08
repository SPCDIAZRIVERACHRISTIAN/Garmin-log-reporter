import datetime
import json
import logging
import os
import sys
import readchar
import requests
from contextlib import suppress
from datetime import timedelta
from getpass import getpass
from pathlib import Path
from typing import Any

from garth.exc import GarthException, GarthHTTPError

from garminconnect import (
    Garmin,
    GarminConnectAuthenticationError,
    GarminConnectConnectionError,
    GarminConnectTooManyRequestsError,
)

logging.getLogger("garminconnect").setLevel(logging.CRITICAL)

api: Garmin | None = None


def safe_readkey() -> str:
    """Safe wrapper around readchar.readkey() that handles non-TTY environments.

    This is particularly useful on macOS and in CI/CD environments where stdin
    might not be a TTY, which would cause readchar to fail with:
    termios.error: (25, 'Inappropriate ioctl for device')

    Returns:
        str: A single character input from the user

    """
    if not sys.stdin.isatty():
        print("WARNING: stdin is not a TTY. Falling back to input().")
        user_input = input("Enter a key (then press Enter): ")
        return user_input[0] if user_input else ""
    try:
        return readchar.readkey()
    except Exception as e:
        print(f"readkey() failed: {e}")
        user_input = input("Enter a key (then press Enter): ")
        return user_input[0] if user_input else ""


class Config:
    """Configuration class for the Garmin Connect API demo."""

    def __init__(self):
        # Load environment variables
        self.email = os.getenv("EMAIL")
        self.password = os.getenv("PASSWORD")
        self.tokenstore = os.getenv("GARMINTOKENS") or "~/.garminconnect"
        self.tokenstore_base64 = (
            os.getenv("GARMINTOKENS_BASE64") or "~/.garminconnect_base64"
        )

        # Date settings
        self.today = datetime.date.today()
        self.week_start = self.today - timedelta(days=7)
        self.month_start = self.today - timedelta(days=30)

        # API call settings
        self.default_limit = 100
        self.start = 0
        self.start_badge = 1  # Badge related calls start counting at 1

        # Activity settings
        self.activitytype = ""  # Possible values: cycling, running, swimming, multi_sport, fitness_equipment, hiking, walking, other
        self.activityfile = "test_data/*.gpx"  # Supported file types: .fit .gpx .tcx
        self.workoutfile = "test_data/sample_workout.json"  # Sample workout JSON file

        # Export settings
        self.export_dir = Path("your_data")
        self.export_dir.mkdir(exist_ok=True)


# Initialize configuration
config = Config()


def get_mfa() -> str:
    """Get MFA token."""
    return input("MFA one-time code: ")


class DataExporter:
    """Utilities for exporting data in various formats."""

    @staticmethod
    def save_json(data: Any, filename: str, pretty: bool = True) -> str:
        """Save data as JSON file."""
        filepath = config.export_dir / f"{filename}.json"
        with open(filepath, "w", encoding="utf-8") as f:
            if pretty:
                json.dump(data, f, indent=4, default=str, ensure_ascii=False)
            else:
                json.dump(data, f, default=str, ensure_ascii=False)
        return str(filepath)


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


def call_and_display(
    api_method=None,
    *args,
    method_name: str | None = None,
    api_call_desc: str | None = None,
    group_name: str | None = None,
    api_responses: list | None = None,
    **kwargs,
):
    """Unified wrapper that calls API methods safely and displays results.
    Can handle both single API calls and grouped API responses.

    For single API calls:
        call_and_display(api.get_user_summary, "2024-01-01")

    For grouped responses:
        call_and_display(group_name="User Data", api_responses=[("api.get_user", data)])

    Args:
        api_method: The API method to call (for single calls)
        *args: Positional arguments for the API method
        method_name: Human-readable name for the API method (optional)
        api_call_desc: Description for display purposes (optional)
        group_name: Name for grouped display (when displaying multiple responses)
        api_responses: List of (api_call_desc, result) tuples for grouped display
        **kwargs: Keyword arguments for the API method

    Returns:
        For single calls: tuple: (success: bool, result: Any)
        For grouped calls: None

    """
    # Handle grouped display mode
    if group_name is not None and api_responses is not None:
        return _display_group(group_name, api_responses)

    # Handle single API call mode
    if api_method is None:
        raise ValueError(
            "Either api_method or (group_name + api_responses) must be provided"
        )

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
        _display_single(api_call_desc, result)
        return True, result
    # Display error in a consistent format
    _display_single(f"{api_call_desc} [ERROR]", {"error": error_msg})
    return False, None


def _display_single(api_call: str, output: Any):
    """Internal function to display single API response."""
    print(f"\n📡 API Call: {api_call}")
    print("-" * 50)

    if output is None:
        print("No data returned")
        # Save empty JSON to response.json in the export directory
        response_file = config.export_dir / "response.json"
        with open(response_file, "w", encoding="utf-8") as f:
            f.write(f"{'-' * 20} {api_call} {'-' * 20}\n{{}}\n{'-' * 77}\n")
        return

    try:
        # Format the output
        if isinstance(output, int | str | dict | list):
            formatted_output = json.dumps(output, indent=2, default=str)
        else:
            formatted_output = str(output)

        # Save to response.json in the export directory
        response_content = (
            f"{'-' * 20} {api_call} {'-' * 20}\n{formatted_output}\n{'-' * 77}\n"
        )

        response_file = config.export_dir / "response.json"
        with open(response_file, "w", encoding="utf-8") as f:
            f.write(response_content)

        print(formatted_output)
        print("-" * 77)

    except Exception as e:
        print(f"Error formatting output: {e}")
        print(output)


def _display_group(group_name: str, api_responses: list[tuple[str, Any]]):
    """Internal function to display grouped API responses."""
    print(f"\n📡 API Group: {group_name}")

    # Collect all responses for saving
    all_responses = {}
    response_content_parts = []

    for api_call, output in api_responses:
        print(f"\n📋 {api_call}")
        print("-" * 50)

        if output is None:
            print("No data returned")
            formatted_output = "{}"
        else:
            try:
                if isinstance(output, int | str | dict | list):
                    formatted_output = json.dumps(output, indent=2, default=str)
                else:
                    formatted_output = str(output)
                print(formatted_output)
            except Exception as e:
                print(f"Error formatting output: {e}")
                formatted_output = str(output)
                print(output)

        # Store for grouped response file
        all_responses[api_call] = output
        response_content_parts.append(
            f"{'-' * 20} {api_call} {'-' * 20}\n{formatted_output}"
        )
        print("-" * 50)

    # Save grouped responses to file
    try:
        response_file = config.export_dir / "response.json"
        header = "=" * 20 + f" {group_name} " + "=" * 20
        footer = "=" * 77
        content_lines = [header, *response_content_parts, footer, ""]
        grouped_content = "\n".join(content_lines)
        with response_file.open("w", encoding="utf-8") as f:
            f.write(grouped_content)

        print(f"\n✅ Grouped responses saved to: {response_file}")
        print("=" * 77)

    except Exception as e:
        print(f"Error saving grouped responses: {e}")


def format_timedelta(td):
    minutes, seconds = divmod(td.seconds + td.days * 86400, 60)
    hours, minutes = divmod(minutes, 60)
    return f"{hours:d}:{minutes:02d}:{seconds:02d}"


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
