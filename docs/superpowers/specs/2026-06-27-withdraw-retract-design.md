# Design: Withdraw Request Retraction (User Cancel)

**Date:** 2026-06-27
**Status:** Approved
**Scope:** Backend only — additive DB migration, new DB method, new API endpoint, one admin guard update

---

## Problem

Users can submit a withdrawal request but have no way to cancel it before an admin processes it. If a user submits the wrong amount or changes their mind, they must contact support and wait. The gap also creates a race-condition window: if a user wants to retract and an admin simultaneously approves, the approval wins silently.

## Decision

Let users cancel their own `Pending` withdrawal requests via a `DELETE` endpoint. No wallet balance changes (submission never debits; cancellation never credits). A new `Cancelled` enum variant is added to the DB so the admin approval path can detect and reject a concurrent cancel gracefully.

---

## Design

### New `WithdrawStatus` value

Add `Cancelled` to the existing `WithdrawStatus` Postgres enum via an additive migration:

```sql
ALTER TYPE withdraw_status ADD VALUE IF NOT EXISTS 'Cancelled';
```

No columns change. No existing rows are affected.

### New DB method: `WithdrawRequest::cancel_by_user`

**File:** `crates/db_schema/src/impls/withdraw_request.rs`

**Signature:**
```rust
pub async fn cancel_by_user(
    pool: &mut DbPool<'_>,
    id: WithdrawRequestId,
    caller_local_user_id: LocalUserId,
) -> FastJobResult<()>
```

**Logic:**
1. Fetch the row by `id`. Return `FastJobErrorType::NotFound` if missing.
2. Ownership check: if `row.local_user_id != caller_local_user_id`, return `FastJobErrorType::NotFound` (avoids leaking existence to other users).
3. Status guard: only `Pending` can be cancelled. `Completed`, `Rejected`, or `Cancelled` return `FastJobErrorType::InvalidField("This withdrawal request cannot be cancelled")`.
4. Update `status = Cancelled`, `updated_at = now()`.
5. Return `Ok(())`.

No transaction needed — this is a single-row update with no wallet movement.

### Admin guard update

**File:** `crates/api/api/src/admin/wallet.rs`, inside `admin_withdraw_wallet_inner`

The existing `match locked.status` has arms for `Pending`, `Completed`, `Rejected`. Add:

```rust
WithdrawStatus::Cancelled => {
    return Err(
        FastJobErrorType::InvalidField(
            "This withdraw request was cancelled by the user".to_string(),
        )
        .into(),
    );
}
```

This is race-safe: the `lock_for_approval_on_conn` (`SELECT FOR UPDATE`) serializes concurrent cancel + admin-approve on the same row. Whichever commits first wins; the other sees the updated status and returns an error.

### New API handler: `retract_withdraw`

**File:** `crates/api/api/src/local_user/withdraw.rs`

**Endpoint:** `DELETE /api/v4/account/wallet/withdraw-requests/{id}`

**Logic:**
```rust
pub async fn retract_withdraw(
    path: Path<WithdrawRequestId>,
    context: Data<FastJobContext>,
    local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
    let id = path.into_inner();
    WithdrawRequest::cancel_by_user(
        &mut context.pool(),
        id,
        local_user_view.local_user.id,
    )
    .await?;
    Ok(Json(SuccessResponse::default()))
}
```

### Route registration

**File:** `src/api_routes.rs`

Add inside the existing `/withdraw-requests` scope:

```rust
.route("/{id}", delete().to(retract_withdraw))
```

---

## What does NOT change

- Wallet balances (no debit at submit-time, no credit at cancel-time)
- Admin approval and reject flows (except the new Cancelled arm in the match)
- Request/response shapes for existing endpoints
- Auth checks (endpoint requires authenticated `LocalUserView` as all user endpoints do)

---

## Files touched

| File | Change |
|------|--------|
| `migrations/2026-06-27-000000-withdraw-status-cancelled/up.sql` | `ALTER TYPE withdraw_status ADD VALUE IF NOT EXISTS 'Cancelled'` |
| `migrations/2026-06-27-000000-withdraw-status-cancelled/down.sql` | No-op (Postgres cannot remove enum values) |
| `crates/db_schema_file/src/enums.rs` | Add `Cancelled` variant to `WithdrawStatus` |
| `crates/db_schema/src/impls/withdraw_request.rs` | Add `cancel_by_user()` + 3 DB tests |
| `crates/api/api/src/admin/wallet.rs` | Add `Cancelled` arm to `match locked.status` |
| `crates/api/api/src/local_user/withdraw.rs` | Add `retract_withdraw` handler |
| `src/api_routes.rs` | Register `DELETE /{id}` route |

---

## Testing

Three DB tests in `withdraw_request.rs` (reusing existing `make_user` fixture):

1. `cancel_pending_sets_cancelled` — creates `Pending` request, calls `cancel_by_user`, asserts `status == Cancelled`.
2. `cancel_non_pending_is_rejected` — creates request, marks it `Completed`, calls `cancel_by_user`, asserts `InvalidField` error.
3. `cancel_other_users_request_is_forbidden` — creates request for user A, calls `cancel_by_user` as user B, asserts `NotFound` error.
