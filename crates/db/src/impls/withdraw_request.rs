#[cfg(feature = "full")]
use crate::schema::withdraw_requests;
use crate::{
  newtypes::WithdrawRequestId,
  source::withdraw_request::{
    WithdrawRequest,
    WithdrawRequestInsertForm,
    WithdrawRequestUpdateForm,
  },
};
#[cfg(feature = "full")]
use crate::{
  traits::Crud,
  utils::{get_conn, DbPool},
};
#[cfg(feature = "full")]
use app_108jobs_core::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use diesel::ExpressionMethods;
#[cfg(feature = "full")]
use diesel::QueryDsl;
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;

#[cfg(feature = "full")]
impl Crud for WithdrawRequest {
  type InsertForm = WithdrawRequestInsertForm;
  type UpdateForm = WithdrawRequestUpdateForm;
  type IdType = WithdrawRequestId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(withdraw_requests::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(withdraw_requests::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}

#[cfg(feature = "full")]
impl WithdrawRequest {
  /// Update a withdraw request by user_bank_account_id (or another field if needed)
  pub async fn update_by_user_bank_account_id(
    pool: &mut DbPool<'_>,
    user_bank_account_id: i32,
    form: &WithdrawRequestUpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      withdraw_requests::table
        .filter(withdraw_requests::user_bank_account_id.eq(user_bank_account_id)),
    )
    .set(form)
    .get_result::<Self>(conn)
    .await
    .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  /// Fetch all withdraw requests by a user
  pub async fn get_by_user(pool: &mut DbPool<'_>, user_id: i32) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    withdraw_requests::table
      .filter(withdraw_requests::local_user_id.eq(user_id))
      .order(withdraw_requests::created_at.desc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  /// Re-read a `WithdrawRequest` by id with `SELECT ... FOR UPDATE` on the
  /// supplied connection. Intended for use inside a `run_transaction` so
  /// concurrent admin-approval or reject calls serialize on this row and
  /// the `status` check is racing-safe before debiting / changing state.
  pub async fn lock_for_approval_on_conn(
    conn: &mut diesel_async::AsyncPgConnection,
    id: WithdrawRequestId,
  ) -> FastJobResult<Self> {
    withdraw_requests::table
      .find(id)
      .for_update()
      .first::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  /// Set `status` + `reason` for the row identified by `id` on the supplied
  /// connection. Intended for use inside the same `run_transaction` that
  /// holds the `FOR UPDATE` lock from [`Self::lock_for_approval_on_conn`].
  pub async fn set_status_on_conn(
    conn: &mut diesel_async::AsyncPgConnection,
    id: WithdrawRequestId,
    new_status: crate::enums::WithdrawStatus,
    reason: Option<String>,
  ) -> FastJobResult<()> {
    let updated = diesel::update(withdraw_requests::table.find(id))
      .set((
        withdraw_requests::status.eq(new_status),
        withdraw_requests::reason.eq(reason),
        withdraw_requests::updated_at.eq(chrono::Utc::now()),
      ))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;
    if updated == 0 {
      return Err(FastJobErrorType::NotFound.into());
    }
    Ok(())
  }

  /// Cancel a `Pending` withdrawal request on behalf of the owning user.
  ///
  /// Returns `FastJobErrorType::NotFound` if the row doesn't exist OR belongs
  /// to a different user (to avoid leaking existence). Returns
  /// `FastJobErrorType::InvalidField` if the request is not in `Pending` status.
  pub async fn cancel_by_user(
    pool: &mut DbPool<'_>,
    id: WithdrawRequestId,
    caller_local_user_id: crate::newtypes::LocalUserId,
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
    if row.status != crate::enums::WithdrawStatus::Pending {
      return Err(
        FastJobErrorType::InvalidField("This withdrawal request cannot be cancelled".to_string())
          .into(),
      );
    }

    diesel::update(withdraw_requests::table.find(id))
      .set((
        withdraw_requests::status.eq(crate::enums::WithdrawStatus::Cancelled),
        withdraw_requests::updated_at.eq(chrono::Utc::now()),
      ))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)?;

    Ok(())
  }
}

// ============================================================================
// DB-backed tests for the WithdrawRequest CRUD surface used by
// `api/local_user/withdraw::submit_withdraw` and
// `api/admin/wallet::{admin_withdraw_wallet, admin_reject_withdraw_request}`.
//
// Coverage:
//   * Creating a request defaults to Pending
//   * Approval path: Pending -> Completed (admin_withdraw_wallet)
//   * Reject path:   Pending -> Rejected with reason (admin_reject)
//   * get_by_user only returns the calling user's rows (tenant isolation)
//
// NOTE: This layer does NOT debit the user's wallet at submit-time.
// The submit handler in api/local_user/withdraw.rs creates a row and returns
// — actual debit happens later in admin_withdraw_wallet via WalletModel.
// ============================================================================
#[cfg(feature = "full")]
#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    enums::WithdrawStatus,
    newtypes::{BankAccountId, Coin, LocalUserId},
    schema::{banks, local_user},
    source::{
      bank::BankInsertForm,
      currency::Currency,
      instance::Instance,
      person::{Person, PersonInsertForm},
      user_bank_account::{BankAccount, UserBankAccountInsertForm},
    },
    test_data::pool_for_tests,
    traits::Crud,
    utils::get_conn,
  };
  use chrono::Utc;
  use diesel::ExpressionMethods;
  use diesel_async::RunQueryDsl;
  use serial_test::serial;

  struct UserCtx {
    instance_id: crate::newtypes::InstanceId,
    local_user_id: LocalUserId,
    wallet_id: crate::newtypes::WalletId,
    bank_account_id: BankAccountId,
    currency_id: crate::newtypes::CurrencyId,
  }

  async fn make_user(pool: &mut DbPool<'_>, label: &str) -> UserCtx {
    let inst = Instance::read_or_create(
      pool,
      format!("wd-test-{}-{}.tld", label, uuid::Uuid::new_v4()),
    )
    .await
    .expect("create instance");

    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let suffix_short = &suffix[..8];
    let (p_form, wallet) =
      PersonInsertForm::test_form_with_wallet(pool, inst.id, &format!("{label}-{suffix_short}"))
        .await
        .expect("test_form_with_wallet");
    let person = Person::create(pool, &p_form).await.expect("create person");

    let local_user_id: i32 = {
      let conn = &mut get_conn(pool).await.expect("get conn");
      diesel::insert_into(local_user::table)
        .values((
          local_user::person_id.eq(person.id),
          local_user::password_encrypted.eq::<Option<String>>(None),
        ))
        .returning(local_user::id)
        .get_result(conn)
        .await
        .expect("insert local_user")
    };

    // Insert a bank row directly (banks are seeded by migration but we want a
    // self-contained test that doesn't depend on the seed surviving).
    let bank_id: i32 = {
      let conn = &mut get_conn(pool).await.expect("get conn");
      let bank_form = BankInsertForm {
        name: format!("Test Bank {suffix_short}"),
        country_id: "TH".to_string(),
        bank_code: Some(format!("TB{suffix_short}")),
        swift_code: None,
        is_active: Some(true),
      };
      diesel::insert_into(banks::table)
        .values(&bank_form)
        .returning(banks::id)
        .get_result::<i32>(conn)
        .await
        .expect("insert bank")
    };

    let acc = BankAccount::create(
      pool,
      &UserBankAccountInsertForm {
        local_user_id: LocalUserId(local_user_id),
        bank_id: crate::newtypes::BankId(bank_id),
        account_number: "1234567890".to_string(),
        account_name: format!("Account {suffix_short}"),
        verification_image_path: None,
      },
    )
    .await
    .expect("create bank account");

    let currency = Currency::get_default(pool)
      .await
      .expect("get_default")
      .expect("THB seeded");

    UserCtx {
      instance_id: inst.id,
      local_user_id: LocalUserId(local_user_id),
      wallet_id: wallet.id,
      bank_account_id: acc.id,
      currency_id: currency.id,
    }
  }

  fn insert_form(ctx: &UserCtx, amount: i32) -> WithdrawRequestInsertForm {
    WithdrawRequestInsertForm {
      local_user_id: ctx.local_user_id,
      wallet_id: ctx.wallet_id,
      user_bank_account_id: ctx.bank_account_id,
      amount: Coin(amount),
      currency_id: ctx.currency_id,
      amount_currency: amount as f64,
      conversion_rate_used: 1,
      reason: Some("test withdrawal".to_string()),
    }
  }

  async fn cleanup(pool: &mut DbPool<'_>, instance_id: crate::newtypes::InstanceId) {
    let _ = Instance::delete(pool, instance_id).await;
  }

  /// New withdraw requests default to Pending and DO NOT touch wallet balance.
  #[tokio::test]
  #[serial]
  async fn create_defaults_pending() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = make_user(pool, "p").await;

    let created = WithdrawRequest::create(pool, &insert_form(&ctx, 500))
      .await
      .expect("create");
    assert_eq!(created.status, WithdrawStatus::Pending);
    assert_eq!(created.amount.0, 500);
    cleanup(pool, ctx.instance_id).await;
  }

  /// admin_withdraw_wallet transitions Pending -> Completed and stores a reason.
  #[tokio::test]
  #[serial]
  async fn approval_marks_completed() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = make_user(pool, "a").await;
    let created = WithdrawRequest::create(pool, &insert_form(&ctx, 200))
      .await
      .expect("create");

    let updated = WithdrawRequest::update(
      pool,
      created.id,
      &WithdrawRequestUpdateForm {
        status: Some(WithdrawStatus::Completed),
        updated_at: Some(Utc::now()),
        reason: Some(Some("approved by admin".to_string())),
      },
    )
    .await
    .expect("approve");
    assert_eq!(updated.status, WithdrawStatus::Completed);
    assert_eq!(updated.reason.as_deref(), Some("approved by admin"));
    cleanup(pool, ctx.instance_id).await;
  }

  /// admin_reject_withdraw_request transitions Pending -> Rejected.
  #[tokio::test]
  #[serial]
  async fn reject_marks_rejected_with_reason() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = make_user(pool, "r").await;
    let created = WithdrawRequest::create(pool, &insert_form(&ctx, 300))
      .await
      .expect("create");

    let updated = WithdrawRequest::update(
      pool,
      created.id,
      &WithdrawRequestUpdateForm {
        status: Some(WithdrawStatus::Rejected),
        updated_at: Some(Utc::now()),
        reason: Some(Some("insufficient kyc".to_string())),
      },
    )
    .await
    .expect("reject");
    assert_eq!(updated.status, WithdrawStatus::Rejected);
    assert_eq!(updated.reason.as_deref(), Some("insufficient kyc"));
    cleanup(pool, ctx.instance_id).await;
  }

  /// get_by_user must return only the calling user's withdrawal requests.
  /// Guards against cross-tenant leak in `/account/wallet/withdraw-requests`.
  #[tokio::test]
  #[serial]
  async fn get_by_user_isolates_tenants() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let alice = make_user(pool, "alice").await;
    let bob = make_user(pool, "bob").await;

    let _a1 = WithdrawRequest::create(pool, &insert_form(&alice, 100))
      .await
      .expect("alice 1");
    let _a2 = WithdrawRequest::create(pool, &insert_form(&alice, 150))
      .await
      .expect("alice 2");
    let _b1 = WithdrawRequest::create(pool, &insert_form(&bob, 200))
      .await
      .expect("bob 1");

    let alice_rows = WithdrawRequest::get_by_user(pool, alice.local_user_id.0)
      .await
      .expect("get alice rows");
    let bob_rows = WithdrawRequest::get_by_user(pool, bob.local_user_id.0)
      .await
      .expect("get bob rows");

    assert_eq!(alice_rows.len(), 2);
    assert!(alice_rows
      .iter()
      .all(|r| r.local_user_id == alice.local_user_id));
    assert_eq!(bob_rows.len(), 1);
    assert_eq!(bob_rows[0].local_user_id, bob.local_user_id);

    cleanup(pool, alice.instance_id).await;
    cleanup(pool, bob.instance_id).await;
  }

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
    assert!(msg.contains("NotFound"), "expected NotFound, got {msg}");

    cleanup(pool, alice.instance_id).await;
    cleanup(pool, bob.instance_id).await;
  }
}
