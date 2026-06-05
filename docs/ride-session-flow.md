# Ride Session Flow

## TripStatus Enum

```
Pending → Assigned → RiderConfirmed → EnRouteToPickup → PickedUp → EnRouteToDropoff → Delivered
    ↘                                                                        ↘
     ↘ (any status) → Cancelled                                               ↘
```

## Status Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           RIDE SESSION LIFECYCLE                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  1. CREATE RIDE (employer)                                                      │
│     POST /api/v4/rides/create                                                   │
│     ┌─────────┐     rider specified      ┌──────────┐                           │
│     │ Pending │ ────────────────────────→│ Assigned │                           │
│     └─────────┘                           └──────────┘                           │
│          │                                     │                                │
│          │ rider accepts                       │                                │
│          ▼                                     ▼                                │
│     (assignment)                        2. CONFIRM (rider)                      │
│                                        POST /api/v4/rides/{sessionId}/confirm   │
│                                               │                                 │
│                                               ▼                                 │
│                                        ┌───────────────┐                        │
│                                        │RiderConfirmed│                         │
│                                        └───────────────┘                        │
│                                               │                                 │
│  3. UPDATE STATUS (rider)                     │                                 │
│     PUT /api/v4/rides/{sessionId}/status      │                                 │
│                                               ▼                                 │
│                                        ┌───────────────┐                        │
│                                        │EnRouteToPickup│                        │
│                                        └───────────────┘                        │
│                                               │                                 │
│                                               ▼                                 │
│                                        ┌───────────┐                            │
│                                        │ PickedUp │                             │
│                                        └───────────┘                            │
│                                               │                                 │
│  4. UPDATE METER (rider - ongoing)            │                                 │
│     PUT /api/v4/rides/{sessionId}/meter       ▼                                 │
│     (updates price, distance, time)    ┌──────────────┐                        │
│                                        │EnRouteToDropoff│                       │
│                                        └──────────────┘                        │
│                                               │                                 │
│                                               ▼                                 │
│                                        ┌───────────┐                            │
│                                        │ Delivered │                            │
│                                        └───────────┘                            │
│                                               │                                 │
│  5. CONFIRM COMPLETION (employer)             │                                 │
│     POST /api/v4/deliveries/{postId}/confirm  ▼                                 │
│                                        [Funds Released]                        │
│                                                                                  │
│  ─────────────────────────────────────────────────────────────────────────────  │
│                                                                                  │
│  CANCEL (rider or employer) - from any active status                            │
│  POST /api/v4/rides/{sessionId}/cancel                                          │
│  ┌───────────┐                                                                  │
│  │ Cancelled │                                                                  │
│  └───────────┘                                                                  │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## API Endpoints Summary

| Endpoint | Method | Who | Status Change |
|----------|--------|-----|---------------|
| `/rides/create` | POST | Employer | → `Pending` or `Assigned` |
| `/rides/{id}/confirm` | POST | Rider | `Assigned` → `RiderConfirmed` |
| `/rides/{id}/status` | PUT | Rider/Employer | Progresses through statuses |
| `/rides/{id}/meter` | PUT | Rider | Updates price/distance/time |
| `/rides/{id}/cancel` | POST | Rider/Employer | → `Cancelled` |
| `/deliveries/{postId}/confirm` | POST | Employer | Releases payment after `Delivered` |

**Note**: `/deliveries/{postId}/status` is for package deliveries only, NOT ride sessions.

## Status Definitions

| Status | Description |
|--------|-------------|
| `Pending` | Ride requested, no rider assigned yet |
| `Assigned` | Rider has been assigned to the ride |
| `RiderConfirmed` | Rider has confirmed they're taking the ride |
| `EnRouteToPickup` | Rider is heading to pickup location |
| `PickedUp` | Customer/package picked up |
| `EnRouteToDropoff` | Rider is heading to dropoff location |
| `Delivered` | Ride completed successfully |
| `Cancelled` | Ride was cancelled |

## Key Implementation Files

- **Status Enum**: `crates/db_schema_file/src/enums.rs` - `TripStatus`
- **Session Struct**: `crates/db_schema/src/source/ride_session.rs` - `RideSession`
- **Session Methods**: `crates/db_schema/src/impls/ride_session.rs`
- **Create Handler**: `crates/api/api/src/delivery/ride.rs` - `create_ride_session`
- **Confirm Handler**: `crates/api/api/src/delivery/ride.rs` - `confirm_ride_assignment`
- **Meter Handler**: `crates/api/api/src/delivery/ride.rs` - `update_ride_meter`
- **Cancel Handler**: `crates/api/api/src/delivery/ride.rs` - `cancel_ride_session`
- **Status Handler**: `crates/api/api/src/delivery/status.rs` - `update_delivery_status`
- **Completion Handler**: `crates/api/api/src/delivery/confirm.rs` - `confirm_delivery_completion`
- **Routes**: `src/api_routes.rs`

## Key Points

1. **Creation**: Employer creates ride, optionally specifying a rider
2. **Confirmation**: Rider must confirm they're taking the ride
3. **Status Updates**: Rider updates status as they progress (via `/deliveries/{postId}/status`)
4. **Meter Updates**: Rider sends ongoing meter updates for dynamic pricing
5. **Cancellation**: Either party can cancel until `Delivered`
6. **Completion**: Employer confirms delivery to release escrowed funds

## Real-time Events

All status changes are published to Redis for real-time WebSocket notifications via `publish_ride_event`:

```rust
RideStatusEvent {
  kind: "ride_created" | "ride_confirmed" | "ride_cancelled" | "ride_meter_updated" | ...,
  session_id,
  post_id,
  status,
  updated_at,
}
```
