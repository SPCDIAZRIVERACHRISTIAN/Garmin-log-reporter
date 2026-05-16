# Garmin Run Data Specification
Required schema for coaching + analytics compatibility.

---

## REQUIRED FIELDS (Minimum Dataset)

| Field | Type | Unit | Description | Required |
|------|------|------|-------------|----------|
activity_id | string/int | — | Unique activity identifier | ✅ |
start_time | ISO8601 string | UTC | Activity start timestamp | ✅ |
activity_type | string | — | run / trail_run / treadmill | ✅ |
distance_m | float | meters | Total distance | ✅ |
duration_sec | float | seconds | Total moving duration | ✅ |
avg_hr | int | bpm | Average heart rate | ✅ |
max_hr | int | bpm | Max heart rate | ✅ |
calories | int | kcal | Calories burned | ✅ |

---

## STRONGLY RECOMMENDED (Performance Analysis)

| Field | Type | Unit | Description |
|------|------|------|-------------|
avg_cadence | float | spm | Average cadence |
elevation_gain_m | float | meters | Elevation gain |
elevation_loss_m | float | meters | Elevation loss |
avg_pace_sec_per_km | float | sec/km | Average pace |
training_effect_aerobic | float | score | Garmin aerobic load |
training_effect_anaerobic | float | score | Garmin anaerobic load |
terrain | string | — | road / trail / track / treadmill |

---

## SPLITS ARRAY (Highly Valuable)

```json
"splits": [
  {
    "split_index": 1,
    "distance_m": 1000,
    "duration_sec": 320,
    "avg_hr": 150,
    "avg_pace_sec_per_km": 320
  }
]
