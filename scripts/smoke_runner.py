from garmin_client.client import GarminClient
from garmin_client.config import Config
from garmin_client.utils import prompt_mfa_code


def fail(msg):
    print(f"\n❌ SMOKE FAILED: {msg}")
    exit(1)


def ok(msg):
    print(f"✅ {msg}")


print("\n=== SMOKE RUNNER ===\n")

client = GarminClient(Config())


# login
success, _, err = client.login(prompt_mfa_code)
if not success:
    fail(err)

ok("login works")


# fetch activities
success, acts, err = client.get_activities("2024-01-01", "2025-01-01")
if not success:
    fail(err)

if not acts:
    fail("no activities returned")

ok(f"{len(acts)} activities fetched")


# first activity
first = acts[0]

if "activityId" not in first:
    fail("activity missing id")

ok("activity list valid")


# detail call
success, detail, err = client.get_activity(first["activityId"])
if not success:
    fail(err)

if not isinstance(detail, dict):
    fail("detail response invalid")

ok("activity detail valid")


print("\n🚀 SMOKE TEST PASSED\n")
