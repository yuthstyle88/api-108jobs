# Design: Escrow Refund on Delivery Cancellation

**Date:** 2026-06-27
**Status:** Approved
**Scope:** Backend only — no API surface change, no new DB columns

---

## Problem

When a delivery is cancelled after a rider has been assigned, the employer's funds
held in escrow are never returned. The `update_delivery_status` handler calls
`DeliveryDetails::update_status()` which only changes the status column; it has no
knowledge of the wallet. Employers permanently lose the delivery fee on every
cancellation.

## Decision

Full refund to the employer, regardless of how far along the delivery was.
Partial-refund / rider-compensation logic is out of scope for this change.

---

## Design

### Approach

New method `DeliveryDetails::cancel_and_refund_escrow()` in
`crates/db_schema/src/impls/delivery_details.rs`, following the exact same
structure as the existing `assign_from_comment_with_escrow` and
`confirm_completion_and_release_payment` methods.

The HTTP handler in `crates/api/api/src/delivery/status.rs` calls the new method
instead of `update_status()` when the requested status is `Cancelled` and the
delivery is already assigned.

### `DeliveryDetails::cancel_and_refund_escrow(pool, post_id, reason)`

**Signature:**
```rust
pub async fn cancel_and_refund_escrow(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    reason: Option<String>,
) -> FastJobResult<Self>
```

**Logic:**

1. Read the current delivery row (`get_by_post_id`).
2. **Guard** — if `assigned_rider_id.is_none()` or `delivery_fee == 0`, no escrow
   was ever held: delegate to `update_status(pool, post_id, Cancelled, reason)` and
   return.
3. Otherwise, run a single DB transaction:
   a. Resolve employer identity:
      `Post::read()` → `post.creator_id` (PersonId)
      → `LocalUser` (by person_id) → `local_user_id`
      → `WalletModel::get_by_user()` → employer wallet
   b. Build `WalletTransactionInsertForm`:
      - `wallet_id`: employer wallet id
      - `reference_type`: `"delivery"`
      - `reference_id`: `post_id.0`
      - `kind`: `TxKind::Transfer`
      - `amount`: `current_delivery.delivery_fee`
      - `description`: `format!("escrow refund for cancelled delivery: post {}", post_id.0)`
      - `idempotency_key`: `format!("cancel-refund:{}:{}", post_id.0, employer_local_user_id.0)`
   c. Call `WalletModel::refund_from_platform_on_conn(conn, &tx_form)` — moves funds
      platform → employer, journals both sides, no CoinModel change (mirrors the
      `hold()` operation exactly in reverse).
   d. Update delivery status to `Cancelled` and set `cancellation_reason` in the
      same transaction.
4. Return the updated `DeliveryDetails`.

**Idempotency:** The deterministic key `cancel-refund:{post_id}:{employer_local_user_id}`
means retrying the same cancellation is safe — the wallet layer will deduplicate
and the status update is also idempotent (same-status returns early).

### `status.rs` change

```
if new_status == Cancelled && current_delivery.assigned_rider_id.is_some() {
    // Escrow was held — use the refund path
    DeliveryDetails::cancel_and_refund_escrow(&mut pool, post_id, reason).await?
} else {
    DeliveryDetails::update_status(&mut pool, post_id, Cancelled, reason).await?
}
```

No new endpoint, no request/response shape change.

---

## What does NOT change

- DB schema (no new columns)
- API surface (`PUT /api/v4/deliveries/{postId}/status`)
- Request/response types
- Auth checks (already enforced before this code runs)
- CoinModel balance (escrow refund is internal wallet movement, not coin issuance)

---

## Files touched

| File | Change |
|------|--------|
| `crates/db_schema/src/impls/delivery_details.rs` | Add `cancel_and_refund_escrow()` |
| `crates/api/api/src/delivery/status.rs` | Branch on `Cancelled + assigned` to call new method |

---

## Testing

- Unit/DB test in `delivery_details.rs` (existing `#[cfg(test)]` block):
  `cancel_assigned_delivery_refunds_employer_wallet` — assigns a delivery (holds
  escrow), cancels it, asserts employer wallet balance returns to original value and
  delivery status is `Cancelled`.
- Existing state-machine tests (`can_transition_to_*`) are unaffected.
