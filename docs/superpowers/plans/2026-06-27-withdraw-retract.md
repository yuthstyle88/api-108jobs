# Withdraw Request Retraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let authenticated users cancel their own `Pending` withdrawal requests via `DELETE /api/v4/account/wallet/withdraw-requests/{id}`.

**Architecture:** Additive Postgres enum migration → new `cancel_by_user` DB method (ownership + status guard, single UPDATE) → new `Cancelled` arm in the admin approval match (race-safety) → new `retract_withdraw` Actix handler wired into the existing `/withdraw-requests` scope.

**Tech Stack:** Rust, Diesel async, PostgreSQL, Actix-web

## Global Constraints

- Never touch the default branch; always work on a feature branch.
- Additive migration only — `ALTER TYPE ... ADD VALUE`. Never drop or rename the enum or existing values.
- No wallet balance change on cancel (submit never debits; cancel never credits).
- Ownership check returns `FastJobErrorType::NotFound` (not `UnauthorizedAccess`) to avoid leaking row existence to other users.
- Run tests with: `app_108jobs_CONFIG_LOCATION=/Users/koeyl/108-ecosystem/108jobs/api-108jobs/config/config.ci.hjson app_108jobs_DATABASE_URL=postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1 cargo test --features full -p app_108jobs_db_schema`
- Run `RUSTFMT="$(rustup which --toolchain nightly rustfmt)" cargo +nightly fmt --all` before every commit.

---

## File Map

| File | Change |
|------|--------|
| `migrations/2026-06-27-000005-0000_withdraw_status_cancelled/up.sql` | New — `ALTER TYPE` |
| `migrations/2026-06-27-000005-0000_withdraw_status_cancelled/down.sql` | New — no-op comment |
| `crates/db_schema_file/src/enums.rs` | Add `Cancelled` variant to `WithdrawStatus` |
| `crates/db_schema/src/impls/withdraw_request.rs` | Add `cancel_by_user()` + 3 DB tests |
| `crates/api/api/src/admin/wallet.rs` | Add `Cancelled` arm to `match locked.status` |
| `crates/api/api/src/local_user/withdraw.rs` | Add `retract_withdraw` handler |
| `src/api_routes.rs` | Register `DELETE /{id}` in the `/withdraw-requests` scope |

---

### Task 1: DB Migration + Enum Variant

**Files:**
- Create: `migrations/2026-06-27-000005-0000_withdraw_status_cancelled/up.sql`
- Create: `migrations/2026-06-27-000005-0000_withdraw_status_cancelled/down.sql`
- Modify: `crates/db_schema_file/src/enums.rs`

**Interfaces:**
- Produces: `WithdrawStatus::Cancelled` variant usable in all downstream crates

---

- [ ] **Step 1: Create the feature branch**

```bash
git checkout -b feat/withdraw-retract
```

- [ ] **Step 2: Create the migration directory and SQL files**

```bash
mkdir -p migrations/2026-06-27-000005-0000_withdraw_status_cancelled
```

Write `migrations/2026-06-27-000005-0000_withdraw_status_cancelled/up.sql`:
```sql
-- Add Cancelled to the withdraw_status enum.
-- Postgres cannot remove enum values, so the down migration is intentionally a no-op.
ALTER TYPE withdraw_status ADD VALUE IF NOT EXISTS 'Cancelled';
```

Write `migrations/2026-06-27-000005-0000_withdraw_status_cancelled/down.sql`:
```sql
-- Postgres does not support removing enum values; this migration cannot be reversed.
-- Existing Cancelled rows would need to be back-filled to Pending before a clean rollback.
SELECT 1;
```

- [ ] **Step 3: Add `Cancelled` variant to `WithdrawStatus` in `crates/db_schema_file/src/enums.rs`**

Find the `WithdrawStatus` enum (around line 361). It currently reads:

```rust
pub enum WithdrawStatus {
  /// Pending
  #[default]
  Pending,
  /// Rejected
  Rejected,
  /// Completed
  Completed,
}
```

Replace with:

```rust
pub enum WithdrawStatus {
  /// Pending
  #[default]
  Pending,
  /// Rejected
  Rejected,
  /// Completed
  Completed,
  /// Cancelled by the user before admin processing
  Cancelled,
}
```

- [ ] **Step 4: Apply the migration to the local dev DB**

```bash
diesel migration run --database-url postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1
```

Expected: `Running migration 2026-06-27-000005-0000_withdraw_status_cancelled`

- [ ] **Step 5: Verify the enum change compiles**

```bash
cargo check --features full -p app_108jobs_db_schema_file 2>&1 | tail -5
```

Expected: `Finished` with no errors.

- [ ] **Step 6: Apply nightly rustfmt**

```bash
RUSTFMT="$(rustup which --toolchain nightly rustfmt)" cargo +nightly fmt --all
```

- [ ] **Step 7: Commit**

```bash
git add migrations/2026-06-27-000005-0000_withdraw_status_cancelled/ \
        crates/db_schema_file/src/enums.rs
git commit -m "feat(withdraw): add Cancelled variant to WithdrawStatus enum"
```

---

### Task 2: `cancel_by_user` DB Method + Tests

**Files:**
- Modify: `crates/db_schema/src/impls/withdraw_request.rs`

**Interfaces:**
- Consumes: `WithdrawStatus::Cancelled` (from Task 1)
- Produces: `WithdrawRequest::cancel_by_user(pool, id, caller_local_user_id) -> FastJobResult<()>`

---

- [ ] **Step 1: Write three failing tests inside the existing `#[cfg(test)] mod tests` block in `withdraw_request.rs`**

The test module already has a `make_user` fixture and `insert_form` helper. Add the following three tests at the end of the `mod tests` block, before its closing `}`:

```rust
/// cancel_by_user on a Pending request sets status to Cancelled.
#[tokio::test]
#[serial]
async fn cancel_pending_sets_cancelled() {
  let pool = pool_for_tests();
  let pool = &mut (&pool).into();
  let ctx = make_user(pool, "cp").await;

  let created = WithdrawRequest::create(pool, &insert_form(&ctx, 400))
    .await
    .expect("create");
  assert_eq!(created.status, WithdrawStatus::Pending);

  WithdrawRequest::cancel_by_user(pool, created.id, ctx.local_user_id)
    .await
    .expect("cancel");

  // Re-fetch and verify status
  let updated = WithdrawRequest::read(pool, created.id)
    .await
    .expect("read after cancel");
  assert_eq!(updated.status, WithdrawStatus::Cancelled);

  cleanup(pool, ctx.instance_id).await;
}

/// cancel_by_user on a non-Pending request must return InvalidField.
#[tokio::test]
#[serial]
async fn cancel_non_pending_is_rejected() {
  use chrono::Utc;

  let pool = pool_for_tests();
  let pool = &mut (&pool).into();
  let ctx = make_user(pool, "cnp").await;

  let created = WithdrawRequest::create(pool, &insert_form(&ctx, 200))
    .await
    .expect("create");

  // Mark it Completed first (simulates already-approved request)
  WithdrawRequest::update(
    pool,
    created.id,
    &WithdrawRequestUpdateForm {
      status: Some(WithdrawStatus::Completed),
      updated_at: Some(Utc::now()),
      reason: None,
    },
  )
  .await
  .expect("mark completed");

  let err = WithdrawRequest::cancel_by_user(pool, created.id, ctx.local_user_id)
    .await
    .expect_err("should fail on non-Pending");

  let msg = format!("{err:?}");
  assert!(
    msg.contains("InvalidField"),
    "expected InvalidField, got {msg}"
  );

  cleanup(pool, ctx.instance_id).await;
}

/// cancel_by_user for a different user's request must return NotFound.
#[tokio::test]
#[serial]
async fn cancel_other_users_request_is_forbidden() {
  let pool = pool_for_tests();
  let pool = &mut (&pool).into();
  let alice = make_user(pool, "alice-c").await;
  let bob = make_user(pool, "bob-c").await;

  let alices_request = WithdrawRequest::create(pool, &insert_form(&alice, 300))
    .await
    .expect("create alice request");

  let err = WithdrawRequest::cancel_by_user(pool, alices_request.id, bob.local_user_id)
    .await
    .expect_err("bob must not cancel alice's request");

  let msg = format!("{err:?}");
  assert!(
    msg.contains("NotFound"),
    "expected NotFound, got {msg}"
  );

  cleanup(pool, alice.instance_id).await;
  cleanup(pool, bob.instance_id).await;
}
```

- [ ] **Step 2: Run the tests to confirm they fail with "method does not exist"**

```bash
app_108jobs_CONFIG_LOCATION=/Users/koeyl/108-ecosystem/108jobs/api-108jobs/config/config.ci.hjson \
app_108jobs_DATABASE_URL=postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1 \
cargo test --features full -p app_108jobs_db_schema \
  cancel_pending_sets_cancelled 2>&1 | tail -5
```

Expected: compile error — `cancel_by_user` does not exist yet.

- [ ] **Step 3: Implement `cancel_by_user` in `crates/db_schema/src/impls/withdraw_request.rs`**

Add the following method to `impl WithdrawRequest` (after `set_status_on_conn`, before the `#[cfg(test)]` block):

```rust
/// Cancel a `Pending` withdrawal request on behalf of the owning user.
///
/// Returns `FastJobErrorType::NotFound` if the row doesn't exist OR belongs
/// to a different user (to avoid leaking existence). Returns
/// `FastJobErrorType::InvalidField` if the request is not in `Pending` status.
pub async fn cancel_by_user(
  pool: &mut DbPool<'_>,
  id: WithdrawRequestId,
  caller_local_user_id: LocalUserId,
) -> FastJobResult<()> {
  use diesel::QueryDsl;

  let conn = &mut get_conn(pool).await?;
  let row = withdraw_requests::table
    .find(id)
    .first::<Self>(conn)
    .await
    .map_err(|_| FastJobErrorType::NotFound)?;

  // Ownership check — returns NotFound to avoid leaking row existence.
  if row.local_user_id != caller_local_user_id {
    return Err(FastJobErrorType::NotFound.into());
  }

  // Status guard — only Pending requests can be retracted.
  if row.status != app_108jobs_db_schema_file::enums::WithdrawStatus::Pending {
    return Err(
      FastJobErrorType::InvalidField(
        "This withdrawal request cannot be cancelled".to_string(),
      )
      .into(),
    );
  }

  diesel::update(withdraw_requests::table.find(id))
    .set((
      withdraw_requests::status
        .eq(app_108jobs_db_schema_file::enums::WithdrawStatus::Cancelled),
      withdraw_requests::updated_at.eq(chrono::Utc::now()),
    ))
    .execute(conn)
    .await
    .with_fastjob_type(FastJobErrorType::DatabaseError)?;

  Ok(())
}
```

- [ ] **Step 4: Run the three new tests and verify they all pass**

```bash
app_108jobs_CONFIG_LOCATION=/Users/koeyl/108-ecosystem/108jobs/api-108jobs/config/config.ci.hjson \
app_108jobs_DATABASE_URL=postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1 \
cargo test --features full -p app_108jobs_db_schema \
  cancel_pending_sets_cancelled cancel_non_pending_is_rejected cancel_other_users_request_is_forbidden 2>&1 | tail -10
```

Expected:
```
test impls::withdraw_request::tests::cancel_pending_sets_cancelled ... ok
test impls::withdraw_request::tests::cancel_non_pending_is_rejected ... ok
test impls::withdraw_request::tests::cancel_other_users_request_is_forbidden ... ok
test result: ok. 3 passed; 0 failed
```

- [ ] **Step 5: Run the full `db_schema` test suite to check for regressions**

```bash
app_108jobs_CONFIG_LOCATION=/Users/koeyl/108-ecosystem/108jobs/api-108jobs/config/config.ci.hjson \
app_108jobs_DATABASE_URL=postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1 \
cargo test --features full -p app_108jobs_db_schema 2>&1 | tail -5
```

Expected: all tests pass (count will be ≥ 70 after adding the 3 new ones).

- [ ] **Step 6: Apply nightly rustfmt**

```bash
RUSTFMT="$(rustup which --toolchain nightly rustfmt)" cargo +nightly fmt --all
```

- [ ] **Step 7: Commit**

```bash
git add crates/db_schema/src/impls/withdraw_request.rs
git commit -m "feat(withdraw): add cancel_by_user with ownership and status guards"
```

---

### Task 3: Admin Approval Guard Update

**Files:**
- Modify: `crates/api/api/src/admin/wallet.rs`

**Interfaces:**
- Consumes: `WithdrawStatus::Cancelled` (from Task 1)

The `admin_withdraw_wallet_inner` function contains a `match locked.status` block that currently has three arms: `Pending`, `Completed`, `Rejected`. Adding `Cancelled` makes the match exhaustive and prevents a silent approval of a user-cancelled request.

---

- [ ] **Step 1: Add `Cancelled` arm to the match in `admin_withdraw_wallet_inner`**

Find the match block (around line 271). It currently reads:

```rust
match locked.status {
  WithdrawStatus::Pending => {}
  WithdrawStatus::Completed => {
    return Err::<(Coin, Coin), app_108jobs_utils::error::FastJobError>(
      FastJobErrorType::InvalidField(
        "This withdraw request has already been processed".to_string(),
      )
      .into(),
    );
  }
  WithdrawStatus::Rejected => {
    return Err(
      FastJobErrorType::InvalidField(
        "This withdraw request was rejected and cannot be approved".to_string(),
      )
      .into(),
    );
  }
}
```

Replace with:

```rust
match locked.status {
  WithdrawStatus::Pending => {}
  WithdrawStatus::Completed => {
    return Err::<(Coin, Coin), app_108jobs_utils::error::FastJobError>(
      FastJobErrorType::InvalidField(
        "This withdraw request has already been processed".to_string(),
      )
      .into(),
    );
  }
  WithdrawStatus::Rejected => {
    return Err(
      FastJobErrorType::InvalidField(
        "This withdraw request was rejected and cannot be approved".to_string(),
      )
      .into(),
    );
  }
  WithdrawStatus::Cancelled => {
    return Err(
      FastJobErrorType::InvalidField(
        "This withdraw request was cancelled by the user".to_string(),
      )
      .into(),
    );
  }
}
```

- [ ] **Step 2: Compile check**

```bash
cargo check --features full -p app_108jobs_api 2>&1 | tail -5
```

Expected: `Finished` with no errors.

- [ ] **Step 3: Apply nightly rustfmt**

```bash
RUSTFMT="$(rustup which --toolchain nightly rustfmt)" cargo +nightly fmt --all
```

- [ ] **Step 4: Commit**

```bash
git add crates/api/api/src/admin/wallet.rs
git commit -m "feat(withdraw): guard admin approval against Cancelled requests"
```

---

### Task 4: API Handler + Route Registration

**Files:**
- Modify: `crates/api/api/src/local_user/withdraw.rs`
- Modify: `src/api_routes.rs`

**Interfaces:**
- Consumes: `WithdrawRequest::cancel_by_user(pool, id, caller_local_user_id)` (from Task 2)
- Produces: `DELETE /api/v4/account/wallet/withdraw-requests/{id}` → `retract_withdraw`

---

- [ ] **Step 1: Add the `retract_withdraw` handler to `crates/api/api/src/local_user/withdraw.rs`**

Add the following import at the top of the file (merge into the existing `app_108jobs_db_schema` import block):

```rust
use app_108jobs_db_schema::newtypes::WithdrawRequestId;
```

Then add the handler at the end of the file:

```rust
pub async fn retract_withdraw(
  path: actix_web::web::Path<WithdrawRequestId>,
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

- [ ] **Step 2: Register the route in `src/api_routes.rs`**

Add `retract_withdraw` to the import line (around line 111):

```rust
withdraw::{list_withdraw_requests, retract_withdraw, submit_withdraw},
```

Find the `/withdraw-requests` scope block (around line 488):

```rust
scope("/withdraw-requests")
  // GET /wallet/withdraw-requests → list withdrawal requests
  .route("", get().to(list_withdraw_requests))
  // POST /wallet/withdraw-requests → create new withdrawal request
  .route("", post().to(submit_withdraw)),
```

Replace with:

```rust
scope("/withdraw-requests")
  // GET /wallet/withdraw-requests → list withdrawal requests
  .route("", get().to(list_withdraw_requests))
  // POST /wallet/withdraw-requests → create new withdrawal request
  .route("", post().to(submit_withdraw))
  // DELETE /wallet/withdraw-requests/{id} → user retracts a pending request
  .route("/{id}", delete().to(retract_withdraw)),
```

- [ ] **Step 3: Compile check the full workspace**

```bash
cargo check --features full -p app_108jobs_api 2>&1 | tail -5
```

Expected: `Finished` with no errors.

- [ ] **Step 4: Apply nightly rustfmt**

```bash
RUSTFMT="$(rustup which --toolchain nightly rustfmt)" cargo +nightly fmt --all
```

- [ ] **Step 5: Commit**

```bash
git add crates/api/api/src/local_user/withdraw.rs src/api_routes.rs
git commit -m "feat(withdraw): add DELETE endpoint for user retraction of pending requests"
```

---

### Task 5: Push and Open PR

- [ ] **Step 1: Push the branch**

```bash
git push -u origin feat/withdraw-retract
```

- [ ] **Step 2: Open the PR**

```bash
gh pr create \
  --title "feat(withdraw): let users retract pending withdrawal requests" \
  --body "$(cat <<'EOF'
## Summary

- Adds `Cancelled` to the `WithdrawStatus` Postgres enum (additive migration).
- Adds `WithdrawRequest::cancel_by_user()` — ownership check (returns NotFound
  for wrong user to avoid leaking existence) + status guard (only Pending can
  be cancelled) + single UPDATE, no wallet balance change.
- Guards `admin_withdraw_wallet_inner` against approving an already-cancelled
  request (race-safe via the existing SELECT FOR UPDATE lock pattern).
- New endpoint: `DELETE /api/v4/account/wallet/withdraw-requests/{id}`.

## Test plan

- [x] `cancel_pending_sets_cancelled` — Pending → Cancelled
- [x] `cancel_non_pending_is_rejected` — Completed → InvalidField error
- [x] `cancel_other_users_request_is_forbidden` — wrong user → NotFound error
- [x] All existing db_schema tests still pass
- [ ] CI green

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

---

## Self-Review

**Spec coverage:**
- ✅ `Cancelled` enum value — Task 1
- ✅ `cancel_by_user` with ownership + status guard — Task 2
- ✅ Admin guard update — Task 3
- ✅ `DELETE /wallet/withdraw-requests/{id}` endpoint — Task 4
- ✅ 3 DB tests (pending→cancelled, non-pending rejected, wrong-user forbidden) — Task 2
- ✅ No wallet balance change anywhere

**Placeholder scan:** None found.

**Type consistency:**
- `WithdrawRequest::cancel_by_user(pool, id: WithdrawRequestId, caller_local_user_id: LocalUserId) -> FastJobResult<()>` — same signature in Task 2 definition and Task 4 call site.
- `WithdrawStatus::Cancelled` — same variant name in Task 1 (enum), Task 2 (DB update), Task 3 (admin match arm).
- Route path `/{id}` with extractor `Path<WithdrawRequestId>` — consistent between Task 4 handler and route registration.
