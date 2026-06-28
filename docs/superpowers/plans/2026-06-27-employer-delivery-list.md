# Employer Delivery List & Single Ride Detail Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add two authenticated employer-scoped endpoints — `GET /api/v4/account/deliveries` (list) and `GET /api/v4/account/deliveries/{postId}` (single) — that return only the calling employer's deliveries in the full private shape.

**Architecture:** DB layer adds two new methods on `DeliveryDetails`: `list_by_employer` (inner JOIN with `post` on `creator_id`) and `get_by_post_id_for_employer` (two sequential reads with ownership check). API layer adds two handlers in `delivery/list.rs` and registers them under `/account/deliveries` in `api_routes.rs`.

**Tech Stack:** Rust/Diesel async, Actix-web, PostgreSQL. No migrations.

## Global Constraints

- Run `cargo +nightly fmt --all` before every commit — CI uses nightly rustfmt with `wrap_comments = true`
- Test command: `app_108jobs_CONFIG_LOCATION=/Users/koeyl/108-ecosystem/108jobs/api-108jobs/config/config.hjson app_108jobs_DATABASE_URL=postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1 cargo test --features full -p app_108jobs_db_schema`
- Never break existing tests — run the full suite before committing
- No new migrations — this is read-only query logic only
- Return `NotFound` (not `UnauthorizedAccess`) for wrong-owner checks — avoids leaking row existence
- Branch off `main`, never commit directly to `main`

---

## File Map

| File | Change |
|------|--------|
| `crates/db_schema/src/impls/delivery_details.rs` | Add `list_by_employer` + `get_by_post_id_for_employer` + 2 tests |
| `crates/api/api/src/delivery/list.rs` | Add `list_employer_deliveries` + `get_employer_delivery` handlers |
| `src/api_routes.rs` | Import new handlers; add `/account/deliveries` scope |

---

## Task 1: DB methods — `list_by_employer` + `get_by_post_id_for_employer`

**Files:**
- Modify: `crates/db_schema/src/impls/delivery_details.rs` (add after `get_all_cancelled`, before `#[cfg(test)]`)

**Interfaces:**
- Produces:
  - `DeliveryDetails::list_by_employer(pool: &mut DbPool<'_>, employer_person_id: PersonId) -> FastJobResult<Vec<Self>>`
  - `DeliveryDetails::get_by_post_id_for_employer(pool: &mut DbPool<'_>, post_id: PostId, employer_person_id: PersonId) -> FastJobResult<Self>`
- Both consumed by Task 2.

**Context you need before coding:**
- All needed imports are already at the top of the file: `PersonId`, `PostId`, `Post`, `get_conn`, `FastJobErrorType`, `FastJobErrorExt`, `delivery_details`, `post as post_tbl`
- `joinable!(delivery_details -> post (post_id))` is declared in the schema — `.inner_join(post_tbl::dsl::post)` works without `.on()`
- `delivery_details::all_columns` selects only the delivery_details columns (needed when joining)
- `post.creator_id` is type `PersonId`; comparison `post.creator_id != employer_person_id` works directly

- [ ] **Step 1: Write the two failing tests**

Add at the bottom of the `#[cfg(test)] mod tests` block (inside it, after the last existing test):

```rust
  /// list_by_employer returns only the deliveries whose post was created
  /// by the given employer PersonId, ignoring all other employers' deliveries.
  #[tokio::test]
  #[serial]
  async fn list_by_employer_returns_own_only() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();

    // Two employers, each with one delivery.
    let (inst1, pid1, _) = fixture_with_status(pool, TripStatus::Pending).await;
    let (inst2, pid2, _) = fixture_with_status(pool, TripStatus::Assigned).await;

    let post1 = Post::read(pool, pid1).await.expect("read post1");
    let post2 = Post::read(pool, pid2).await.expect("read post2");

    let list1 = DeliveryDetails::list_by_employer(pool, post1.creator_id)
      .await
      .expect("list for employer 1");
    let list2 = DeliveryDetails::list_by_employer(pool, post2.creator_id)
      .await
      .expect("list for employer 2");

    let ids1: Vec<PostId> = list1.iter().map(|d| d.post_id).collect();
    let ids2: Vec<PostId> = list2.iter().map(|d| d.post_id).collect();

    assert!(ids1.contains(&pid1), "employer 1 must see own delivery");
    assert!(!ids1.contains(&pid2), "employer 1 must NOT see employer 2's delivery");
    assert!(ids2.contains(&pid2), "employer 2 must see own delivery");
    assert!(!ids2.contains(&pid1), "employer 2 must NOT see employer 1's delivery");

    cleanup(pool, inst1).await;
    cleanup(pool, inst2).await;
  }

  /// get_by_post_id_for_employer returns Ok when the caller owns the delivery,
  /// and NotFound when they don't.
  #[tokio::test]
  #[serial]
  async fn get_by_post_id_for_employer_rejects_wrong_owner() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();

    let (inst1, pid1, _) = fixture_with_status(pool, TripStatus::Pending).await;
    let post1 = Post::read(pool, pid1).await.expect("read post1");
    let owner_id = post1.creator_id;

    // Create a second employer (different person) to obtain a different PersonId.
    let (inst2, pid2, _) = fixture_with_status(pool, TripStatus::Pending).await;
    let post2 = Post::read(pool, pid2).await.expect("read post2");
    let other_id = post2.creator_id;

    // Owner can fetch their own delivery.
    let result = DeliveryDetails::get_by_post_id_for_employer(pool, pid1, owner_id)
      .await
      .expect("owner must be able to fetch their delivery");
    assert_eq!(result.post_id, pid1);

    // Different person gets NotFound.
    let err = DeliveryDetails::get_by_post_id_for_employer(pool, pid1, other_id)
      .await
      .expect_err("wrong owner must get NotFound");
    assert!(
      format!("{err:?}").contains("NotFound"),
      "expected NotFound, got {err:?}"
    );

    cleanup(pool, inst1).await;
    cleanup(pool, inst2).await;
  }
```

Also add `Post` to the test imports (it's needed for `Post::read` inside the tests). Find the existing `use super::*;` and add the Post import if not already present — the test module uses `use super::*;` which imports `Post` via the impl file's top-level `use crate::source::post::Post` import. No change needed.

- [ ] **Step 2: Run tests to verify they fail (compile error expected)**

```bash
cd /Users/koeyl/108-ecosystem/108jobs/api-108jobs
app_108jobs_CONFIG_LOCATION=/Users/koeyl/108-ecosystem/108jobs/api-108jobs/config/config.hjson \
  app_108jobs_DATABASE_URL=postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1 \
  cargo test --features full -p app_108jobs_db_schema \
  list_by_employer_returns_own_only 2>&1 | tail -20
```

Expected: compile error — `no method named list_by_employer found`.

- [ ] **Step 3: Implement `list_by_employer` and `get_by_post_id_for_employer`**

Add these two methods immediately before the `#[cfg(test)]` block (i.e., after `get_all_cancelled`):

```rust
  /// List all deliveries created by the given employer (post.creator_id = employer_person_id).
  /// Returns all statuses, ordered by created_at descending.
  pub async fn list_by_employer(
    pool: &mut DbPool<'_>,
    employer_person_id: PersonId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    delivery_details::dsl::delivery_details
      .inner_join(post_tbl::dsl::post)
      .filter(post_tbl::dsl::creator_id.eq(employer_person_id.0))
      .select(delivery_details::all_columns)
      .order(delivery_details::dsl::created_at.desc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateDeliveryDetails)
  }

  /// Fetch a delivery by post_id, returning NotFound if the post was not
  /// created by employer_person_id (ownership check — does not reveal existence).
  pub async fn get_by_post_id_for_employer(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    employer_person_id: PersonId,
  ) -> FastJobResult<Self> {
    let delivery = Self::get_by_post_id(pool, post_id).await?;
    let post = Post::read(pool, post_id).await?;
    if post.creator_id != employer_person_id {
      return Err(FastJobErrorType::NotFound.into());
    }
    Ok(delivery)
  }
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd /Users/koeyl/108-ecosystem/108jobs/api-108jobs
app_108jobs_CONFIG_LOCATION=/Users/koeyl/108-ecosystem/108jobs/api-108jobs/config/config.hjson \
  app_108jobs_DATABASE_URL=postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1 \
  cargo test --features full -p app_108jobs_db_schema \
  list_by_employer_returns_own_only get_by_post_id_for_employer_rejects_wrong_owner 2>&1 | tail -20
```

Expected: both pass. If the `inner_join` causes a type error, check that `delivery_details::all_columns` is used in the select.

- [ ] **Step 5: Run full DB schema test suite**

```bash
app_108jobs_CONFIG_LOCATION=/Users/koeyl/108-ecosystem/108jobs/api-108jobs/config/config.hjson \
  app_108jobs_DATABASE_URL=postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1 \
  cargo test --features full -p app_108jobs_db_schema 2>&1 | tail -10
```

Expected: all tests pass (previously 68; now 70).

- [ ] **Step 6: Format and commit**

```bash
cd /Users/koeyl/108-ecosystem/108jobs/api-108jobs
cargo +nightly fmt --all
git add crates/db_schema/src/impls/delivery_details.rs
git commit -m "feat(db): add list_by_employer and get_by_post_id_for_employer"
```

---

## Task 2: API handlers — `list_employer_deliveries` + `get_employer_delivery`

**Files:**
- Modify: `crates/api/api/src/delivery/list.rs`

**Interfaces:**
- Consumes:
  - `DeliveryDetails::list_by_employer(pool, PersonId) -> FastJobResult<Vec<DeliveryDetails>>` (from Task 1)
  - `DeliveryDetails::get_by_post_id_for_employer(pool, PostId, PersonId) -> FastJobResult<DeliveryDetails>` (from Task 1)
  - `DeliveryDetails::to_private(&self) -> DeliveryDetailsPrivate` (already exists)
- Produces:
  - `pub async fn list_employer_deliveries(context, local_user_view) -> FastJobResult<Json<Vec<DeliveryDetailsPrivate>>>`
  - `pub async fn get_employer_delivery(path, context, local_user_view) -> FastJobResult<Json<DeliveryDetailsPrivate>>`
  - Both consumed by Task 3.

**Context you need before coding:**

Current `list.rs` imports:
```rust
use actix_web::web::{Data, Json, Path};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::{newtypes::PostId, source::delivery_details::DeliveryDetails};
use app_108jobs_utils::error::FastJobResult;
```

You need to add:
- `DeliveryDetailsPrivate` to the `delivery_details` import
- `LocalUserView` from `app_108jobs_db_views_local_user`

- [ ] **Step 1: Update imports in `list.rs`**

Replace the existing imports at the top of `crates/api/api/src/delivery/list.rs` with:

```rust
use actix_web::web::{Data, Json, Path};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::{
  newtypes::PostId,
  source::delivery_details::{DeliveryDetails, DeliveryDetailsPrivate},
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_utils::error::FastJobResult;
```

- [ ] **Step 2: Add the two new handlers at the end of `list.rs`**

Append after the existing `get_delivery` function:

```rust
/// GET /api/v4/account/deliveries
///
/// Returns all deliveries owned by the authenticated employer (post.creator_id = caller),
/// all statuses, ordered by created_at descending. Returns the full private shape.
pub async fn list_employer_deliveries(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<Vec<DeliveryDetailsPrivate>>> {
  let deliveries =
    DeliveryDetails::list_by_employer(&mut context.pool(), local_user_view.person.id).await?;
  Ok(Json(deliveries.into_iter().map(|d| d.to_private()).collect()))
}

/// GET /api/v4/account/deliveries/{postId}
///
/// Returns a single delivery owned by the authenticated employer.
/// Returns 404 if the delivery does not exist or is not owned by the caller.
pub async fn get_employer_delivery(
  path: Path<PostId>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<DeliveryDetailsPrivate>> {
  let post_id = path.into_inner();
  let delivery = DeliveryDetails::get_by_post_id_for_employer(
    &mut context.pool(),
    post_id,
    local_user_view.person.id,
  )
  .await?;
  Ok(Json(delivery.to_private()))
}
```

- [ ] **Step 3: Verify the crate compiles**

```bash
cd /Users/koeyl/108-ecosystem/108jobs/api-108jobs
cargo check --features full -p app_108jobs_api 2>&1 | tail -20
```

Expected: no errors.

- [ ] **Step 4: Format and commit**

```bash
cargo +nightly fmt --all
git add crates/api/api/src/delivery/list.rs
git commit -m "feat(api): add list_employer_deliveries and get_employer_delivery handlers"
```

---

## Task 3: Route registration

**Files:**
- Modify: `src/api_routes.rs`

**Interfaces:**
- Consumes:
  - `list_employer_deliveries` (from Task 2)
  - `get_employer_delivery` (from Task 2)
- Produces: `GET /api/v4/account/deliveries` and `GET /api/v4/account/deliveries/{postId}` routes live and routable

**Context you need before coding:**

In `src/api_routes.rs`, the delivery list import block currently reads:
```rust
delivery::{
    assign::assign_delivery_from_proposal,
    confirm::confirm_delivery_completion,
    list::{
        get_active_deliveries,
        get_cancelled_deliveries,
        get_completed_deliveries,
        get_delivery,
    },
    ...
```

The `/account` scope has `/wallet`, `/banks`, `/services`, and others. Add `/deliveries` after `/banks`.

- [ ] **Step 1: Extend the delivery list import**

Find this block in `src/api_routes.rs`:
```rust
    list::{
      get_active_deliveries,
      get_cancelled_deliveries,
      get_completed_deliveries,
      get_delivery,
    },
```

Replace with:
```rust
    list::{
      get_active_deliveries,
      get_cancelled_deliveries,
      get_completed_deliveries,
      get_delivery,
      get_employer_delivery,
      list_employer_deliveries,
    },
```

- [ ] **Step 2: Add the `/account/deliveries` scope**

Find this line in the `/account` scope:
```rust
            // Bank account management scope
            .service(scope("/banks").route("", get().to(list_banks)))
```

Add the employer deliveries scope immediately after it:
```rust
            // Bank account management scope
            .service(scope("/banks").route("", get().to(list_banks)))
            // Employer delivery list + single ride detail
            .service(
              scope("/deliveries")
                .route("", get().to(list_employer_deliveries))
                .route("/{postId}", get().to(get_employer_delivery)),
            )
```

- [ ] **Step 3: Verify the whole app compiles**

```bash
cd /Users/koeyl/108-ecosystem/108jobs/api-108jobs
cargo check --features full -p app_108jobs 2>&1 | tail -20
```

Expected: no errors.

- [ ] **Step 4: Run full DB schema test suite one more time**

```bash
app_108jobs_CONFIG_LOCATION=/Users/koeyl/108-ecosystem/108jobs/api-108jobs/config/config.hjson \
  app_108jobs_DATABASE_URL=postgres://postgres:ibrowe123@localhost:5432/fastwork_db_phase1 \
  cargo test --features full -p app_108jobs_db_schema 2>&1 | tail -10
```

Expected: all 70 tests pass, no regressions.

- [ ] **Step 5: Format and commit**

```bash
cargo +nightly fmt --all
git add src/api_routes.rs
git commit -m "feat(routes): register GET /account/deliveries and GET /account/deliveries/{postId}"
```
