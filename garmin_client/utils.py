import sys
import readchar


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


def format_timedelta(td):
    minutes, seconds = divmod(td.seconds + td.days * 86400, 60)
    hours, minutes = divmod(minutes, 60)
    return f"{hours:d}:{minutes:02d}:{seconds:02d}"


def prompt_mfa_code():
    while True:
        code = input("Enter Garmin MFA code: ").strip()
        if code:
            return code
        print("Code cannot be empty.")
