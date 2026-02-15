import logging
from garmin_client.client import GarminClient
from garmin_client.config import Config
from garmin_client.utils import prompt_mfa_code


logging.basicConfig(level=logging.INFO, format="%(levelname)s | %(message)s")


def fail(msg):
    print(f"\n❌ FAILURE: {msg}")
    exit(1)


def success(msg):
    print(f"✅ {msg}")


print("\n==== GARMIN LIVE CHECK ====\n")

config = Config()
client = GarminClient(config)


# ------------------------
# LOGIN TEST
# ------------------------

success_login, _, err = client.login(prompt_mfa_code)

if not success_login:
    fail(f"Login failed → {err}")

success("Login works")


# ------------------------
# FETCH ACTIVITIES
# ------------------------

ok, activities, err = client.get_activities("2024-01-01", "2025-01-01")

if not ok:
    fail(f"Fetching activities failed → {err}")

if not isinstance(activities, list):
    fail("Activities response is not a list")

if not activities:
    fail("No activities returned")

success(f"Fetched {len(activities)} activities")


# ------------------------
# FETCH FIRST ACTIVITY DETAIL
# ------------------------

first_id = activities[0].get("activityId")

if not first_id:
    fail("Activity missing activityId")

ok, detail, err = client.get_activity(first_id)

if not ok:
    fail(f"Activity detail failed → {err}")

if not isinstance(detail, dict):
    fail("Activity detail is not dict")

success("Activity detail fetch works")


# ------------------------
# STRUCTURE CHECK
# ------------------------

required_keys = ["activityId", "activityName"]

missing = [k for k in required_keys if k not in detail]

if missing:
    fail(f"Missing keys in detail response: {missing}")

success("Response structure looks valid")


print("\n🎉 ALL SYSTEMS GO — CLIENT IS WORKING CORRECTLY\n")
