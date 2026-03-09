# Ride Session Sample Walkthrough (Creation → Delivery)

This document traces one concrete ride session end-to-end and shows the exact fare math across multiple meter updates, matching the implementation in `crates/api/api/src/delivery/ride.rs` (`update_ride_meter`).

## Assumed PricingConfig

- currency: USD (minor unit = cents)
- base_fare_coin: 250 (=$2.50)
- minimum_charge_minutes: 3 (time billed in 3‑minute blocks)
- time_charge_per_minute_coin: 20 (=$0.20/min)
- distance_charge_per_km_coin: 150 (=$1.50/km)

At each meter update, the server recalculates the total and publishes a `ride:meter:{session_id}` event.

## 1) Create ride session

- Employer calls `POST /api/v4/rides/create`
- Initial price = `base_fare_coin = 250` ($2.50)
- Event: `ride:status:{sessionId}` with `kind: "ride_requested"` (or `ride_assigned` if pre‑assigned)

State snapshot:

```
current_price_coin = 250 ($2.50)
```

## 2) Rider confirms assignment

- Rider calls `POST /api/v4/rides/{sessionId}/confirm`
- Status becomes `RiderConfirmed`
- Event: `ride:status:{sessionId}` with `kind: "ride_status_update"`, status=`RiderConfirmed`

Price is unchanged until meter updates begin.

## 3) Status moves to pickup → picked up

- Rider calls `PUT /api/v4/rides/{sessionId}/status` to transition through:
  - `EnRouteToPickup` → timestamp set `arrived_at_pickup_at`
  - `PickedUp` → timestamp set `ride_started_at`
- Each step publishes a status event.

## 4) Ongoing meter updates (the money math)

Formula used in `update_ride_meter`:

```
time_blocks = max(1, floor(elapsed_minutes / minimum_charge_minutes))
time_charge_coin = time_blocks * time_charge_per_minute_coin
distance_charge_coin = distance_km * distance_charge_per_km_coin
total_coin = base_fare_coin + time_charge_coin + distance_charge_coin
```

Notes:
- Time is charged by whole blocks of `minimum_charge_minutes`. Partial blocks within the current reading round down, but there is always at least 1 block once any time passes.
- Distance is multiplied directly (floating km × per‑km), then cast to integer coins by the implementation.

### Update A

- Request: `PUT /api/v4/rides/{sessionId}/meter` with `elapsed_minutes=4`, `distance_km=1.2`
- Compute:
  - `time_blocks = max(1, floor(4 / 3)) = 1`
  - `time_charge_coin = 1 × 20 = 20` ($0.20)
  - `distance_charge_coin = 1.2 × 150 = 180` ($1.80)
  - `total_coin = 250 + 20 + 180 = 450` ($4.50)
- Persisted to session:
  - `current_price_coin=450`, `total_distance_km=1.2`, `total_duration_minutes=4`
- Event: `ride:meter:{sessionId}` with `current_price_coin=450`, `elapsed_minutes=4`, `distance_km=1.2`

### Update B

- Request: `elapsed_minutes=9`, `distance_km=3.7`
- Compute:
  - `time_blocks = max(1, floor(9 / 3)) = 3`
  - `time_charge_coin = 3 × 20 = 60` ($0.60)
  - `distance_charge_coin = 3.7 × 150 = 555` ($5.55)
  - `total_coin = 250 + 60 + 555 = 865` ($8.65)
- Persist → Event publishes with `current_price_coin=865`, `elapsed_minutes=9`, `distance_km=3.7`

### Update C

- Request: `elapsed_minutes=17`, `distance_km=6.4`
- Compute:
  - `time_blocks = max(1, floor(17 / 3)) = 5`
  - `time_charge_coin = 5 × 20 = 100` ($1.00)
  - `distance_charge_coin = 6.4 × 150 = 960` ($9.60)
  - `total_coin = 250 + 100 + 960 = 1310` ($13.10)
- Persist → Event publishes with `current_price_coin=1310`, `elapsed_minutes=17`, `distance_km=6.4`

### Update D (final just before drop‑off)

- Request: `elapsed_minutes=22`, `distance_km=8.1`
- Compute:
  - `time_blocks = max(1, floor(22 / 3)) = 7`
  - `time_charge_coin = 7 × 20 = 140` ($1.40)
  - `distance_charge_coin = 8.1 × 150 = 1215` ($12.15)
  - `total_coin = 250 + 140 + 1215 = 1605` ($16.05)
- Persist → Event publishes with `current_price_coin=1605`, `elapsed_minutes=22`, `distance_km=8.1`

## 5) Status → EnRouteToDropoff → Delivered

- Rider calls status updates. On `Delivered`, `ride_completed_at` is set and a status event is published.
- The last meter total on record is the ride’s price reference (`current_price_coin=1605`).

## 6) Employer confirms completion (funds release)

- Employer calls `POST /api/v4/deliveries/{postId}/confirm`
- Escrowed funds are released using the final recorded meter amount.

## Recap of the price over time

- Create: $2.50
- Update A (4 min, 1.2 km): $4.50
- Update B (9 min, 3.7 km): $8.65
- Update C (17 min, 6.4 km): $13.10
- Update D (22 min, 8.1 km): $16.05

## References

- Handler: `update_ride_meter` in `crates/api/api/src/delivery/ride.rs`
- Fields: `PricingConfig::{base_fare_coin, minimum_charge_minutes, time_charge_per_minute_coin, distance_charge_per_km_coin}`
- Session fields updated: `current_price_coin`, `total_distance_km`, `total_duration_minutes`
- Events: `publish_meter_event` to `ride:meter:{session_id}` and status changes via `publish_ride_event`
