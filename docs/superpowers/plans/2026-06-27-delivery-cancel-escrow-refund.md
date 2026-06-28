# Delivery Cancellation Escrow Refund Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** When a delivery is cancelled after a rider has been assigned, automatically refund the full delivery fee from platform escrow back to the employer's wallet in the same DB transaction as the status change.

**Architecture:** New `DeliveryDetails::cancel_and_refund_escrow()` method in the DB layer follows the same pattern as the existing `confirm_completion_and_release_payment()`. The HTTP handler in `status.rs` branches to this method when the target status is `Cancelled` and a rider is already assigned. No API surface or schema changes.

**Tech Stack:** Rust, Diesel async, PostgreSQL, `diesel_async::scoped_futures::ScopedFutureExt`

## Global Constraints

- Never touch the default branch (main/master); always work on a feature branch.
- No new DB columns, no migration.
- No API surface change — same endpoint, same request/response types.
- Money operations must be atomic with status changes (single transaction).
- Idempotency key for the refund transaction: `format!("cancel-refund:{}:{}", post_id.0, employer_local_user_id.0)`.
- Follow the exact same code patterns as `confirm_completion_and_release_payment` in `delivery_details.rs`.
- Run tests with: `app_108jobs_CONFIG_LOCATION=/path/to/config/config.ci.hjson app_108jobs_DATABASE_URL=postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1 cargo test --features full -p app_108jobs_db_schema`

---

## File Map

| File | Change |
|------|--------|
| `crates/db_schema/src/impls/delivery_details.rs` | Add `cancel_and_refund_escrow()` method + 2 DB tests |
| `crates/api/api/src/delivery/status.rs` | Branch on `Cancelled + assigned` to call new method |

---

### Task 1: Add `cancel_and_refund_escrow()` to `delivery_details.rs` with tests

**Files:**
- Modify: `crates/db_schema/src/impls/delivery_details.rs`

**Interfaces:**
- Produces: `pub async fn cancel_and_refund_escrow(pool: &mut DbPool<'_>, post_id: PostId, reason: Option<String>) -> FastJobResult<DeliveryDetails>`

---

- [ ] **Step 1: Create the feature branch**

```bash
git checkout -b feat/delivery-cancel-escrow-refund
```

- [ ] **Step 2: Write two failing tests in `delivery_details.rs`**

Add inside the existing `#[cfg(test)] mod tests { ... }` block at the bottom of `crates/db_schema/src/impls/delivery_details.rs`, after the last test:

```rust
/// cancel_and_refund_escrow on an unassigned delivery must not touch the
/// wallet and must still transition status to Cancelled.
#[tokio::test]
#[serial]
async fn cancel_unassigned_delivery_skips_refund() {
  let pool = pool_for_tests();
  let pool = &mut (&pool).into();
  let (instance_id, post_id, _) = fixture_with_status(pool, TripStatus::Pending).await;

  let result = DeliveryDetails::cancel_and_refund_escrow(
    pool,
    post_id,
    Some("no rider, just cancel".to_string()),
  )
  .await
  .expect("cancel unassigned delivery");

  assert_eq!(result.status, TripStatus::Cancelled);
  assert_eq!(
    result.cancellation_reason.as_deref(),
    Some("no rider, just cancel")
  );
  cleanup(pool, instance_id).await;
}

/// cancel_and_refund_escrow on an assigned delivery must refund the
/// delivery_fee to the employer wallet and transition status to Cancelled.
#[tokio::test]
#[serial]
async fn cancel_assigned_delivery_refunds_employer_wallet() {
  use crate::source::wallet::WalletModel;
  use app_108jobs_db_schema_file::schema::{delivery_details as dd, local_user as lu};

  let pool = pool_for_tests();
  let pool = &mut (&pool).into();

  // --- fixture: instance + employer person (with wallet) ---
  let inst =
    Instance::read_or_create(pool, format!("dd-refund-{}.tld", uuid::Uuid::new_v4()))
      .await
      .expect("create instance");
  let suffix = uuid::Uuid::new_v4().simple().to_string();
  let (p_form, employer_wallet) =
    PersonInsertForm::test_form_with_wallet(pool, inst.id, &format!("er-{}", &suffix[..8]))
      .await
      .expect("test_form_with_wallet");
  let emp = Person::create(pool, &p_form).await.expect("create employer");

  // --- create a delivery post ---
  let post_id: i32 = {
    let conn = &mut get_conn(pool).await.expect("get conn");
    let mut post_form = PostInsertForm::new("refund test".into(), emp.id);
    post_form.post_kind = Some(PostKind::Delivery);
    diesel::insert_into(post::table)
      .values(&post_form)
      .returning(post::id)
      .get_result(conn)
      .await
      .expect("insert post")
  };
  let post_id = PostId(post_id);

  // --- create delivery row with delivery_fee and a fake assigned_rider_id ---
  const DELIVERY_FEE: i64 = 500;
  let mut form = DeliveryDetailsInsertForm::new(
    post_id,
    "1 Pickup St".to_string(),
    "2 Dropoff St".to_string(),
  );
  form.status = Some(TripStatus::Assigned);
  let delivery = DeliveryDetails::create(pool, &form).await.expect("create delivery");

  // Patch delivery_fee and assigned_rider_id directly (bypass escrow hold for unit test)
  {
    let conn = &mut get_conn(pool).await.expect("get conn");
    // assigned_rider_id just needs to be non-None; use id=1 as a placeholder
    // (the refund path only checks `.is_some()`, it does not validate the rider)
    diesel::update(dd::table.filter(dd::post_id.eq(post_id.0)))
      .set((
        dd::delivery_fee.eq(DELIVERY_FEE),
        dd::assigned_rider_id.eq(delivery.id.0), // any non-None i32 works
      ))
      .execute(conn)
      .await
      .expect("patch delivery");
  }

  // Record employer wallet balance before cancellation
  let wallet_before = WalletModel::get_by_user(pool, {
    let conn = &mut get_conn(pool).await.expect("get conn");
    lu::table
      .filter(lu::person_id.eq(emp.id.0))
      .select(lu::id)
      .first::<i32>(conn)
      .await
      .map(crate::newtypes::LocalUserId)
      .expect("employer local_user")
  })
  .await
  .expect("get employer wallet");

  // --- act ---
  let cancelled = DeliveryDetails::cancel_and_refund_escrow(
    pool,
    post_id,
    Some("rider cancelled".to_string()),
  )
  .await
  .expect("cancel with refund");

  // --- assert delivery state ---
  assert_eq!(cancelled.status, TripStatus::Cancelled);
  assert_eq!(
    cancelled.cancellation_reason.as_deref(),
    Some("rider cancelled")
  );

  // --- assert employer wallet received the refund ---
  let employer_local_user_id = {
    let conn = &mut get_conn(pool).await.expect("get conn");
    lu::table
      .filter(lu::person_id.eq(emp.id.0))
      .select(lu::id)
      .first::<i32>(conn)
      .await
      .map(crate::newtypes::LocalUserId)
      .expect("employer local_user")
  };
  let wallet_after = WalletModel::get_by_user(pool, employer_local_user_id)
    .await
    .expect("get employer wallet after");

  assert_eq!(
    wallet_after.balance_available - wallet_before.balance_available,
    DELIVERY_FEE,
    "employer wallet should be credited with the delivery fee"
  );

  let _ = Instance::delete(pool, inst.id).await;
}
```

- [ ] **Step 3: Run the tests to confirm they fail with "not found" / "method does not exist"**

```bash
app_108jobs_CONFIG_LOCATION=/Users/koeyl/108-ecosystem/108jobs/api-108jobs/config/config.ci.hjson \
app_108jobs_DATABASE_URL=postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1 \
cargo test --features full -p app_108jobs_db_schema \
  cancel_unassigned_delivery_skips_refund \
  cancel_assigned_delivery_refunds_employer_wallet 2>&1 | tail -20
```

Expected: compile error — `cancel_and_refund_escrow` does not exist yet.

- [ ] **Step 4: Implement `cancel_and_refund_escrow`**

Add the following method to `impl DeliveryDetails` in `crates/db_schema/src/impls/delivery_details.rs`, directly after `confirm_completion_and_release_payment`:

```rust
/// Cancel a delivery and refund any held escrow back to the employer.
///
/// If the delivery has no assigned rider or zero delivery_fee (no escrow
/// was ever held), delegates to `update_status` — no wallet changes.
///
/// Otherwise runs a single transaction:
///   1. Resolves employer wallet via `post.creator_id → local_user → wallet`.
///   2. Calls `WalletModel::refund_from_platform_on_conn` to move the
///      delivery_fee from platform back to employer (reverses the hold).
///   3. Sets `status = Cancelled` and `cancellation_reason` in the same tx.
pub async fn cancel_and_refund_escrow(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    reason: Option<String>,
) -> FastJobResult<Self> {
    use diesel::QueryDsl;

    // Cheap guard outside the transaction
    let current = Self::get_by_post_id(pool, post_id).await?;
    if current.assigned_rider_id.is_none() || current.delivery_fee == 0 {
        return Self::update_status(pool, post_id, TripStatus::Cancelled, reason).await;
    }

    let delivery_fee = current.delivery_fee;
    let conn = &mut get_conn(pool).await?;

    conn
        .run_transaction(|conn| {
            async move {
                let mut pool: DbPool<'_> = conn.into();

                // Resolve employer: post.creator_id -> local_user -> wallet
                let post = Post::read(&mut pool, post_id).await?;
                let employer_person_id = post.creator_id;

                let conn2 = &mut get_conn(&mut pool).await?;
                let employer_lu = local_user_tbl::dsl::local_user
                    .filter(local_user_tbl::dsl::person_id.eq(employer_person_id.0))
                    .first::<LocalUser>(conn2)
                    .await
                    .map_err(|_| FastJobErrorType::NotFound)?;
                let employer_local_user_id = employer_lu.id;

                let employer_wallet =
                    WalletModel::get_by_user(&mut pool, employer_local_user_id).await?;

                // Refund escrow: platform -> employer wallet
                let tx_form = WalletTransactionInsertForm {
                    wallet_id: employer_wallet.id,
                    reference_type: "delivery".to_string(),
                    reference_id: post_id.0,
                    kind: TxKind::Transfer,
                    amount: delivery_fee,
                    description: format!(
                        "escrow refund for cancelled delivery: post {}",
                        post_id.0
                    ),
                    counter_user_id: Some(employer_local_user_id),
                    // Deterministic: retrying the same cancellation is idempotent.
                    idempotency_key: format!(
                        "cancel-refund:{}:{}",
                        post_id.0, employer_local_user_id.0
                    ),
                };

                let conn3 = &mut get_conn(&mut pool).await?;
                WalletModel::refund_from_platform_on_conn(conn3, &tx_form).await?;

                // Update delivery status + reason in the same transaction
                let conn4 = &mut get_conn(&mut pool).await?;
                let updated = update(
                    delivery_details::dsl::delivery_details
                        .filter(delivery_details::dsl::post_id.eq(post_id.0)),
                )
                .set((
                    delivery_details::dsl::status.eq(TripStatus::Cancelled),
                    delivery_details::dsl::cancellation_reason.eq(reason),
                    delivery_details::dsl::updated_at.eq(Utc::now()),
                ))
                .get_result::<Self>(conn4)
                .await
                .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryDetails)?;

                Ok::<_, app_108jobs_utils::error::FastJobError>(updated)
            }
            .scope_boxed()
        })
        .await
}
```

- [ ] **Step 5: Run the tests and verify they pass**

```bash
app_108jobs_CONFIG_LOCATION=/Users/koeyl/108-ecosystem/108jobs/api-108jobs/config/config.ci.hjson \
app_108jobs_DATABASE_URL=postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1 \
cargo test --features full -p app_108jobs_db_schema \
  cancel_unassigned_delivery_skips_refund \
  cancel_assigned_delivery_refunds_employer_wallet 2>&1 | tail -10
```

Expected:
```
test impls::delivery_details::tests::cancel_unassigned_delivery_skips_refund ... ok
test impls::delivery_details::tests::cancel_assigned_delivery_refunds_employer_wallet ... ok
test result: ok. 2 passed; 0 failed
```

- [ ] **Step 6: Run the full db_schema test suite to check for regressions**

```bash
app_108jobs_CONFIG_LOCATION=/Users/koeyl/108-ecosystem/108jobs/api-108jobs/config/config.ci.hjson \
app_108jobs_DATABASE_URL=postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1 \
cargo test --features full -p app_108jobs_db_schema 2>&1 | tail -10
```

Expected: all existing tests still pass.

- [ ] **Step 7: Commit**

```bash
git add crates/db_schema/src/impls/delivery_details.rs
git commit -m "feat(delivery): add cancel_and_refund_escrow with DB tests"
```

---

### Task 2: Update `status.rs` to call `cancel_and_refund_escrow` for assigned deliveries

**Files:**
- Modify: `crates/api/api/src/delivery/status.rs`

**Interfaces:**
- Consumes: `DeliveryDetails::cancel_and_refund_escrow(pool, post_id, reason) -> FastJobResult<DeliveryDetails>` (from Task 1)

---

- [ ] **Step 1: Add the import for `DeliveryDetails` cancel method (already imported — verify)**

Open `crates/api/api/src/delivery/status.rs`. The import at line 3 already has:
```rust
use app_108jobs_db_schema::{newtypes::PostId, source::delivery_details::DeliveryDetails};
```
No import changes needed.

- [ ] **Step 2: Replace the `update_status` call with the branching logic**

In `crates/api/api/src/delivery/status.rs`, replace the block:

```rust
  // Update the delivery status
  let updated_delivery = {
    let mut pool = context.pool();
    DeliveryDetails::update_status(&mut pool, post_id, new_status, data.reason.clone()).await?
  };
```

with:

```rust
  // Update the delivery status — if cancelling an assigned delivery, also refund escrow.
  let updated_delivery = {
    let mut pool = context.pool();
    if new_status == TripStatus::Cancelled && current_delivery.assigned_rider_id.is_some() {
      DeliveryDetails::cancel_and_refund_escrow(&mut pool, post_id, data.reason.clone()).await?
    } else {
      DeliveryDetails::update_status(&mut pool, post_id, new_status, data.reason.clone()).await?
    }
  };
```

- [ ] **Step 3: Check that the api crate compiles**

```bash
cargo check --features full -p app_108jobs_api 2>&1 | tail -5
```

Expected: `Finished` with no errors.

- [ ] **Step 4: Apply nightly rustfmt**

```bash
RUSTFMT="$(rustup which --toolchain nightly rustfmt)" cargo +nightly fmt --all -- --check 2>&1
```

If there are diffs, apply them:

```bash
RUSTFMT="$(rustup which --toolchain nightly rustfmt)" cargo +nightly fmt --all
```

- [ ] **Step 5: Commit**

```bash
git add crates/api/api/src/delivery/status.rs
git commit -m "feat(delivery): refund escrow when cancelling an assigned delivery"
```

---

### Task 3: Push and open PR

- [ ] **Step 1: Push branch**

```bash
git push -u origin feat/delivery-cancel-escrow-refund
```

- [ ] **Step 2: Open PR**

```bash
gh pr create \
  --title "feat(delivery): refund escrow on cancellation of assigned delivery" \
  --body "$(cat <<'EOF'
## Summary

- Adds `DeliveryDetails::cancel_and_refund_escrow()` — single-transaction method
  that refunds the held delivery fee from platform escrow back to the employer
  wallet before setting status to Cancelled.
- `status.rs` now branches to this method when the target status is Cancelled
  and a rider is already assigned; unassigned cancellations continue through the
  existing `update_status()` path.
- Idempotency key `cancel-refund:{post_id}:{employer_local_user_id}` makes
  retried cancellations safe.

## Test plan

- [x] `cancel_unassigned_delivery_skips_refund` — unassigned cancel delegates to `update_status`, no wallet change
- [x] `cancel_assigned_delivery_refunds_employer_wallet` — employer balance increases by `delivery_fee`
- [x] All existing `delivery_details` tests still pass
- [ ] CI green

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

---

## Self-Review

**Spec coverage:**
- ✅ Full refund to employer — implemented via `refund_from_platform_on_conn`
- ✅ Atomic with status change — single `run_transaction`
- ✅ Idempotent — deterministic key `cancel-refund:{post_id}:{employer_local_user_id}`
- ✅ No escrow held → delegate to `update_status` — guard on `assigned_rider_id.is_none() || delivery_fee == 0`
- ✅ No API surface change
- ✅ No new DB columns

**Placeholder scan:** None found.

**Type consistency:**
- `cancel_and_refund_escrow(pool, post_id, reason)` — exact same signature used in both Task 1 (definition) and Task 2 (call site).
- `TripStatus::Cancelled` — same enum variant in both tasks.
- `current_delivery.assigned_rider_id` — `Option<RiderId>`, `.is_some()` guard consistent in both tasks.
