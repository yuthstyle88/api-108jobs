# Design: Employer Delivery List & Single Ride Detail

**Date:** 2026-06-27
**Status:** Approved
**Scope:** Backend only — two new DB methods, two new API handlers, route registration

---

## Problem

The existing `/deliveries/active|completed|cancelled` endpoints return ALL deliveries across every employer with no ownership filter. An authenticated employer has no way to fetch only their own deliveries. The Flutter client needs a scoped list for the employer dashboard and a single-ride detail view.

## Decision

Add two authenticated employer-scoped endpoints under `/api/v4/account/deliveries`:
- `GET /account/deliveries` — list all deliveries for the caller (all statuses, ordered newest first)
- `GET /account/deliveries/{postId}` — get one delivery, `NotFound` if the caller doesn't own it

Both return the full `DeliveryDetailsPrivate` shape (employer is always authorized for their own deliveries). No new migrations or DB schema changes needed.

---

## Design

### New DB method: `DeliveryDetails::list_by_employer`

**File:** `crates/db_schema/src/impls/delivery_details.rs`

**Signature:**
```rust
pub async fn list_by_employer(
    pool: &mut DbPool<'_>,
    employer_person_id: PersonId,
) -> FastJobResult<Vec<Self>>
```

**Logic:**
- JOIN `delivery_details` with `post` on `delivery_details.post_id = post.id`
- Filter: `post.creator_id = employer_person_id`
- Order: `delivery_details.created_at DESC`
- Returns all statuses (Pending, Assigned, all in-progress, Delivered, Cancelled)

### New DB method: `DeliveryDetails::get_by_post_id_for_employer`

**File:** `crates/db_schema/src/impls/delivery_details.rs`

**Signature:**
```rust
pub async fn get_by_post_id_for_employer(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    employer_person_id: PersonId,
) -> FastJobResult<Self>
```

**Logic:**
1. Fetch `DeliveryDetails` by `post_id` — `NotFound` if missing.
2. Fetch `Post` by `post_id` — `NotFound` if missing.
3. If `post.creator_id != employer_person_id` → return `NotFound` (avoids leaking existence to other users).
4. Return the `DeliveryDetails` row.

No transaction needed — two sequential reads.

### New API handlers

**File:** `crates/api/api/src/delivery/list.rs`

**Handler 1: `list_employer_deliveries`**
```rust
pub async fn list_employer_deliveries(
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<Vec<DeliveryDetailsPrivate>>>
```
- Calls `DeliveryDetails::list_by_employer(&mut pool, local_user_view.person.id)`
- Maps each row to `.to_private()`
- Returns `Json(deliveries.into_iter().map(|d| d.to_private()).collect())`

**Handler 2: `get_employer_delivery`**
```rust
pub async fn get_employer_delivery(
    path: Path<PostId>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<DeliveryDetailsPrivate>>
```
- Calls `DeliveryDetails::get_by_post_id_for_employer(&mut pool, post_id, local_user_view.person.id)`
- Returns `Json(delivery.to_private())`

### Route registration

**File:** `src/api_routes.rs`

Add inside the existing `/account` scope (after the `/wallet` service block):

```rust
.service(
    scope("/deliveries")
        .route("", get().to(list_employer_deliveries))
        .route("/{postId}", get().to(get_employer_delivery)),
)
```

---

## Response shape

Both endpoints return `DeliveryDetailsPrivate` (already defined in `crates/db_schema/src/source/delivery_details.rs`), which includes all fields: full contact info (`sender_name`, `sender_phone`, `receiver_name`, `receiver_phone`), `cod_amount`, `cancellation_reason`, wallet transaction IDs.

---

## What does NOT change

- Existing `/deliveries/active|completed|cancelled` endpoints (unchanged, still return all deliveries)
- Existing `/deliveries/{postId}` endpoint (unchanged, no auth gate)
- DB schema / migrations (no changes)
- `DeliveryDetailsPrivate` / `DeliveryDetailsPublic` structs (unchanged)

---

## Files touched

| File | Change |
|------|--------|
| `crates/db_schema/src/impls/delivery_details.rs` | Add `list_by_employer` + `get_by_post_id_for_employer` + 2 DB tests |
| `crates/api/api/src/delivery/list.rs` | Add `list_employer_deliveries` + `get_employer_delivery` handlers |
| `src/api_routes.rs` | Register `GET /account/deliveries` and `GET /account/deliveries/{postId}` |

---

## Testing

Two DB tests in `delivery_details.rs` (reusing `fixture_with_status`):

1. `list_by_employer_returns_own_only` — creates two employers with one delivery each, asserts each sees only their own row.
2. `get_by_post_id_for_employer_rejects_wrong_owner` — creates a delivery for employer A, calls `get_by_post_id_for_employer` as employer B's `PersonId`, asserts `NotFound`.
