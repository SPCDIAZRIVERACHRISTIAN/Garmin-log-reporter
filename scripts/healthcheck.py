import sys
from datetime import date, timedelta
from garmin_client.exporters import DataExporter as de
from garmin_client.client import GarminClient
from garmin_client.config import Config
from garmin_client.utils import init_api, prompt_mfa_code


def main():
    print("\n--- GARMIN CLIENT HEALTHCHECK ---\n")

    # Load config
    config = Config()
    config.ensure_dirs()

    # Create client
    client = GarminClient(config)

    # Login (interactive)
    print("🔐 Authenticating...")

    ok, err = init_api(client, prompt_mfa_code)

    if not ok:
        print(f"❌ Login failed: {err}")
        sys.exit(1)

    print("✅ Login successful\n")

    # Basic API test
    print("📡 Testing API access...")

    try:
        today = date.today()
        yesterday = today - timedelta(days=1)
        test_date = date(2025, 2, 10)

        data = client.get_activities(yesterday.isoformat(), today.isoformat())
        data1 = client.get_activities(test_date.isoformat(), today.isoformat())

        if not data1:
            print(f"❌ API test failed: {err}")
            sys.exit(1)

        de.export_activities(data1)

        print("✅ API reachable")
        print(f"worked but you havent done anything: {data}")
        print(f"📊 Activities returned: {len(data1)}")
        print(f"activities saved")

    except Exception as e:
        print(f"❌ Healthcheck error: {e}")
        sys.exit(1)

    print("\n🎉 Healthcheck passed\n")


if __name__ == "__main__":
    main()
