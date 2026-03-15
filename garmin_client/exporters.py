import json
from typing import Any
from .config import config
from typing import List, Dict
from pathlib import Path
from datetime import datetime


class DataExporter:
    """Utilities for exporting data in various formats."""

    @staticmethod
    def export_activities(activities: List[Dict]) -> None:

        base_dir = Path(config.export_dir) / "activities"

        for act in activities:
            normalized = DataExporter.normalize_activity(act)

            date = normalized["start_time"].split(" ")[0]
            year, month, _ = date.split("-")

            dir_path = base_dir / year / month
            dir_path.mkdir(parents=True, exist_ok=True)

            file_path = dir_path / f"{normalized['activity_id']}.json"

            with open(file_path, "w", encoding="utf-8") as f:
                json.dump(normalized, f, indent=4, ensure_ascii=False)

    @staticmethod
    def normalize_activity(act: Dict) -> Dict:
        return {
            "activity_id": act["activityId"],
            "activity_name": act["activityName"],
            "sport": act["activityType"]["typeKey"],
            "start_time": act["startTimeLocal"],
            "timestamp": act["beginTimestamp"],
            "distance_meters": act["distance"],
            "duration_seconds": act["duration"],
            "avg_hr": act.get("averageHR"),
            "max_hr": act.get("maxHR"),
            "avg_speed": act.get("averageSpeed"),
            "max_speed": act.get("maxSpeed"),
            "calories": act.get("calories"),
            "steps": act.get("steps"),
            "location": act.get("locationName"),
        }
