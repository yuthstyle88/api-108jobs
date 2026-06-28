//! `wallet_hold` ledger operations.
//!
//! Every public method accepts a borrowed connection to keep the caller in
//! control of the surrounding `run_transaction` boundary — these are NEVER
//! meant to be called outside a transaction that also mutates the wallet.

use crate::{
  newtypes::{BillingId, Coin, WalletHoldId, WalletId},
  schema::wallet_hold,
  source::wallet_hold::{
    hold_status,
    HoldStatus,
    WalletHold,
    WalletHoldInsertForm,
    WalletHoldUpdateForm,
  },
};
use app_108jobs_core::error::{FastJobError, FastJobErrorType, FastJobResult};
use chrono::Utc;
use diesel::{
  result::{DatabaseErrorKind, Error as DieselError},
  ExpressionMethods,
  OptionalExtension,
  QueryDsl,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

impl WalletHold {
  /// Insert a new `Active` hold for `(wallet_id, billing_id, amount)`. A second
  /// insert for the same billing collides on `uq_wallet_hold_active_per_billing`
  /// and is mapped to [`FastJobErrorType::DuplicateWalletHold`].
  pub async fn insert_active(
    conn: &mut AsyncPgConnection,
    wallet_id: WalletId,
    billing_id: BillingId,
    amount: Coin,
    idempotency_key: Option<String>,
  ) -> FastJobResult<WalletHold> {
    if amount <= 0 {
      return Err(FastJobErrorType::AmountMustBePositive.into());
    }
    let form = WalletHoldInsertForm {
      wallet_id,
      billing_id,
      amount,
      status: hold_status::ACTIVE.to_string(),
      idempotency_key,
    };
    let res = diesel::insert_into(wallet_hold::table)
      .values(&form)
      .get_result::<WalletHold>(conn)
      .await;
    match res {
      Ok(h) => Ok(h),
      Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _info)) => {
        Err(FastJobErrorType::DuplicateWalletHold.into())
      }
      Err(e) => Err(FastJobError::from(e)),
    }
  }

  /// Look up the currently-active hold for a billing, if any.
  pub async fn find_active_for_billing(
    conn: &mut AsyncPgConnection,
    billing_id: BillingId,
  ) -> FastJobResult<Option<WalletHold>> {
    let row = wallet_hold::table
      .filter(wallet_hold::billing_id.eq(billing_id))
      .filter(wallet_hold::status.eq(hold_status::ACTIVE))
      .first::<WalletHold>(conn)
      .await
      .optional()?;
    Ok(row)
  }

  /// Transition a hold from `Active` to `to_status`. The `WHERE status='Active'`
  /// guard makes the operation idempotent at the SQL level: a second call sees
  /// 0 rows affected and returns `Ok(None)` so callers can no-op.
  ///
  /// Returns:
  ///   - `Ok(Some(_))` if the hold was transitioned by this call
  ///   - `Ok(None)`    if it was already in `to_status` (idempotent re-call)
  ///   - `Err(_)`      on DB error
  pub async fn transition_from_active(
    conn: &mut AsyncPgConnection,
    id: WalletHoldId,
    to_status: HoldStatus,
  ) -> FastJobResult<Option<WalletHold>> {
    let form = WalletHoldUpdateForm {
      status: Some(to_status.as_str().to_string()),
      released_at: Some(Some(Utc::now())),
    };
    let row = diesel::update(
      wallet_hold::table
        .filter(wallet_hold::id.eq(id))
        .filter(wallet_hold::status.eq(hold_status::ACTIVE)),
    )
    .set(&form)
    .get_result::<WalletHold>(conn)
    .await
    .optional()?;
    Ok(row)
  }

  /// Derive the total Active hold amount for a wallet. Used by invariant
  /// checks: `wallet.balance_outstanding` must equal this value.
  pub async fn sum_active_for_wallet(
    conn: &mut AsyncPgConnection,
    wallet_id: WalletId,
  ) -> FastJobResult<i64> {
    use diesel::dsl::sum;
    let total: Option<i64> = wallet_hold::table
      .filter(wallet_hold::wallet_id.eq(wallet_id))
      .filter(wallet_hold::status.eq(hold_status::ACTIVE))
      .select(sum(wallet_hold::amount))
      .first::<Option<i64>>(conn)
      .await?;
    Ok(total.unwrap_or(0))
  }
}

// ============================================================================
// DB-backed concurrency tests.
//
// These require a running Postgres test database. Follow the existing project
// pattern from `crates/db_schema/src/impls/local_user.rs`: each test calls
// `build_db_pool_for_tests()` which panics if no DB is configured. `#[serial]`
// serializes them because they mutate shared instance rows.
//
// What's covered:
//   1. duplicate_active_hold_rejected  - partial unique index fires
//   2. transition_from_active_is_idempotent - second call returns None
//   3. sum_active_for_wallet_arithmetic - ledger sum derivation
//   4. concurrent_insert_one_winner    - tokio::join! race; one Ok, one Err
//   5. released_then_new_active_allowed - status filter on partial unique idx
// ============================================================================
#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    schema::{billing, local_user, post},
    source::{
      instance::Instance,
      person::{Person, PersonInsertForm},
    },
    test_data::pool_for_tests,
    utils::get_conn,
  };
  use diesel::sql_query;
  use diesel_async::RunQueryDsl;
  use serial_test::serial;

  /// Minimal fixture: instance, employer person, employer wallet, post, billing.
  /// Returns (wallet_id, billing_id) and leaves cleanup to the caller (via Instance::delete
  /// CASCADE). Avoids the full freelancer/local_user fixture by inserting rows
  /// directly via SQL where needed — this keeps the test surface contained to the
  /// new wallet_hold + version code paths without dragging the rest of the schema in.
  async fn fixture(
    pool: &mut crate::utils::DbPool<'_>,
  ) -> (WalletId, BillingId, crate::newtypes::InstanceId) {
    use crate::source::post::PostInsertForm;

    let inst =
      Instance::read_or_create(pool, format!("wallet-hold-test-{}.tld", std::process::id()))
        .await
        .expect("create instance");

    // Employer + freelancer persons, each with a fresh wallet via the
    // wallet-aware test fixture. The name suffix is a UUID so multiple
    // tests in the same `cargo test` invocation don't collide on
    // person.name uniqueness within the shared instance.
    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let suffix_short = &suffix[..8];
    let (emp_form, emp_wallet) =
      PersonInsertForm::test_form_with_wallet(pool, inst.id, &format!("emp-{suffix_short}"))
        .await
        .expect("emp test_form_with_wallet");
    let emp_person = Person::create(pool, &emp_form)
      .await
      .expect("create employer person");

    let (frl_form, _frl_wallet) =
      PersonInsertForm::test_form_with_wallet(pool, inst.id, &format!("frl-{suffix_short}"))
        .await
        .expect("frl test_form_with_wallet");
    let frl_person = Person::create(pool, &frl_form)
      .await
      .expect("create freelancer person");

    // Local users for employer + freelancer (billing FKs reference local_user.id).
    let conn = &mut get_conn(pool).await.expect("get conn");
    let emp_local_user_id: i32 = diesel::insert_into(local_user::table)
      .values((
        local_user::person_id.eq(emp_person.id),
        local_user::password_encrypted.eq::<Option<String>>(None),
      ))
      .returning(local_user::id)
      .get_result(conn)
      .await
      .expect("insert employer local_user");
    let frl_local_user_id: i32 = diesel::insert_into(local_user::table)
      .values((
        local_user::person_id.eq(frl_person.id),
        local_user::password_encrypted.eq::<Option<String>>(None),
      ))
      .returning(local_user::id)
      .get_result(conn)
      .await
      .expect("insert freelancer local_user");

    // Minimal post.
    let post_form = PostInsertForm::new("wallet-hold test post".into(), emp_person.id);
    let post_id: i32 = diesel::insert_into(post::table)
      .values(&post_form)
      .returning(post::id)
      .get_result(conn)
      .await
      .expect("insert post");

    // Chat room. billing.room_id has an FK on chat_room.id; we need a unique
    // room id across cargo invocations because chat_room has no FK to instance
    // and therefore isn't cleaned up by cleanup() cascade.
    use crate::{
      source::chat_room::{ChatRoom, ChatRoomInsertForm},
      traits::Crud,
    };
    let room_id_str = format!(
      "wh-test-{}-{}-{}",
      std::process::id(),
      emp_wallet.id.0,
      chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let chat_form = ChatRoomInsertForm {
      id: crate::newtypes::ChatRoomId(room_id_str.clone()),
      room_name: "wallet-hold test room".to_string(),
      created_at: chrono::Utc::now(),
      updated_at: None,
      post_id: None,
      current_comment_id: None,
    };
    let _ = ChatRoom::create(pool, &chat_form).await.expect("chat room");

    // Insert billing directly so we don't depend on api_common / db_views_billing.
    let conn = &mut get_conn(pool).await.expect("get conn");
    let billing_id: i32 = diesel::insert_into(billing::table)
      .values((
        billing::freelancer_id.eq(frl_local_user_id),
        billing::employer_id.eq(emp_local_user_id),
        billing::post_id.eq(post_id),
        billing::amount.eq(100i32),
        billing::description.eq("wallet-hold test billing".to_string()),
        billing::room_id.eq(room_id_str),
      ))
      .returning(billing::id)
      .get_result(conn)
      .await
      .expect("insert billing");

    (emp_wallet.id, BillingId(billing_id), inst.id)
  }

  async fn cleanup(pool: &mut crate::utils::DbPool<'_>, instance_id: crate::newtypes::InstanceId) {
    // Wallet-hold rows reference billing -> post -> person -> instance via ON DELETE CASCADE.
    // Removing the instance unwinds everything we created in `fixture`.
    let _ = Instance::delete(pool, instance_id).await;
    // Also remove the stand-alone test wallets created via create_for_user — they aren't
    // FK-tied to instance and would otherwise pile up. Cheap raw SQL guards by a known
    // pattern but we don't have one; leave them. CI runs against a fresh DB.
    let conn = get_conn(pool).await;
    if let Ok(mut conn) = conn {
      let _ = sql_query("SELECT 1").execute(&mut *conn).await;
    }
  }

  #[tokio::test]
  #[serial]
  async fn duplicate_active_hold_rejected() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (wallet_id, billing_id, instance_id) = fixture(pool).await;
    let conn = &mut get_conn(pool).await.expect("get conn");

    WalletHold::insert_active(conn, wallet_id, billing_id, Coin(50), None)
      .await
      .expect("first insert succeeds");
    let err = WalletHold::insert_active(conn, wallet_id, billing_id, Coin(50), None)
      .await
      .expect_err("second active hold for same billing must be rejected");
    assert!(
      format!("{err:?}").contains("DuplicateWalletHold"),
      "expected DuplicateWalletHold, got: {err:?}"
    );
    // `conn`'s last use is above; NLL releases the &mut DbPool borrow
    // before `cleanup` re-borrows it.
    cleanup(pool, instance_id).await;
  }

  #[tokio::test]
  #[serial]
  async fn transition_from_active_is_idempotent() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (wallet_id, billing_id, instance_id) = fixture(pool).await;
    let conn = &mut get_conn(pool).await.expect("get conn");

    let hold = WalletHold::insert_active(conn, wallet_id, billing_id, Coin(50), None)
      .await
      .expect("insert");
    let first = WalletHold::transition_from_active(conn, hold.id, HoldStatus::Released)
      .await
      .expect("first transition");
    assert!(first.is_some(), "first transition should report the row");
    let second = WalletHold::transition_from_active(conn, hold.id, HoldStatus::Released)
      .await
      .expect("second transition");
    assert!(second.is_none(), "second transition must be a no-op");
    // `conn`'s last use is above; NLL releases the &mut DbPool borrow
    // before `cleanup` re-borrows it.
    cleanup(pool, instance_id).await;
  }

  #[tokio::test]
  #[serial]
  async fn sum_active_for_wallet_arithmetic() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (wallet_id, billing_id_a, instance_id) = fixture(pool).await;
    let conn = &mut get_conn(pool).await.expect("get conn");

    let initial = WalletHold::sum_active_for_wallet(conn, wallet_id)
      .await
      .expect("sum");
    assert_eq!(initial, 0);

    let _ = WalletHold::insert_active(conn, wallet_id, billing_id_a, Coin(30), None)
      .await
      .expect("insert a");
    let after_a = WalletHold::sum_active_for_wallet(conn, wallet_id)
      .await
      .expect("sum");
    assert_eq!(after_a, 30);

    // Releasing the only hold should drop the sum back to zero.
    let h = WalletHold::find_active_for_billing(conn, billing_id_a)
      .await
      .expect("find")
      .expect("present");
    let _ = WalletHold::transition_from_active(conn, h.id, HoldStatus::Released)
      .await
      .expect("transition");
    let after_release = WalletHold::sum_active_for_wallet(conn, wallet_id)
      .await
      .expect("sum");
    assert_eq!(after_release, 0);

    // `conn`'s last use is above; NLL releases the &mut DbPool borrow
    // before `cleanup` re-borrows it.
    cleanup(pool, instance_id).await;
  }

  #[tokio::test]
  #[serial]
  async fn released_then_new_active_allowed() {
    // The partial unique index is gated on status='Active'. Once a hold is
    // Released/Captured, a fresh Active hold for the same billing must be
    // permitted (covers re-quoting flows that we don't currently expose but
    // shouldn't be blocked at the DB level).
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let (wallet_id, billing_id, instance_id) = fixture(pool).await;
    let conn = &mut get_conn(pool).await.expect("get conn");

    let first = WalletHold::insert_active(conn, wallet_id, billing_id, Coin(10), None)
      .await
      .expect("first insert");
    let _ = WalletHold::transition_from_active(conn, first.id, HoldStatus::Released)
      .await
      .expect("transition");
    // After release, a new Active hold for the same billing succeeds.
    let _second = WalletHold::insert_active(conn, wallet_id, billing_id, Coin(20), None)
      .await
      .expect("second insert allowed after release");

    // `conn`'s last use is above; NLL releases the &mut DbPool borrow
    // before `cleanup` re-borrows it.
    cleanup(pool, instance_id).await;
  }

  #[tokio::test]
  #[serial]
  async fn concurrent_insert_one_winner() {
    // Race two inserts targeting the same billing on independent connections.
    // The partial unique index must let exactly one through.
    let pool = pool_for_tests();
    let p = &mut (&pool).into();
    let (wallet_id, billing_id, instance_id) = fixture(p).await;
    let pool = pool_for_tests();

    let mut p1: crate::utils::DbPool<'_> = (&pool).into();
    let mut p2: crate::utils::DbPool<'_> = (&pool).into();

    let h1 = async {
      let c = &mut get_conn(&mut p1).await.expect("conn1");
      WalletHold::insert_active(c, wallet_id, billing_id, Coin(40), None).await
    };
    let h2 = async {
      let c = &mut get_conn(&mut p2).await.expect("conn2");
      WalletHold::insert_active(c, wallet_id, billing_id, Coin(40), None).await
    };
    let (r1, r2) = tokio::join!(h1, h2);

    let oks = [&r1, &r2].iter().filter(|r| r.is_ok()).count();
    let errs = [&r1, &r2].iter().filter(|r| r.is_err()).count();
    assert_eq!(
      oks, 1,
      "exactly one concurrent insert must succeed; r1={r1:?} r2={r2:?}"
    );
    assert_eq!(
      errs, 1,
      "exactly one concurrent insert must fail; r1={r1:?} r2={r2:?}"
    );
    let err_msg = if r1.is_err() {
      format!("{:?}", r1.unwrap_err())
    } else {
      format!("{:?}", r2.unwrap_err())
    };
    assert!(
      err_msg.contains("DuplicateWalletHold"),
      "expected DuplicateWalletHold, got: {err_msg}"
    );

    let p = &mut (&pool).into();
    cleanup(p, instance_id).await;
  }
}
