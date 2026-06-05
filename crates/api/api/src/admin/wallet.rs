use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_api_utils::utils::{
  is_admin, list_top_up_requests_inner, list_withdraw_requests_inner,
};
use app_108jobs_db_schema::newtypes::{Coin, WithdrawRequestId};
use app_108jobs_db_schema::newtypes::{CoinId, LocalUserId, WalletId};
use app_108jobs_db_schema::source::top_up_request::TopUpRequest;
use app_108jobs_db_schema::source::wallet::{TxKind, WalletModel, WalletTransactionInsertForm};
use app_108jobs_db_schema::source::withdraw_request::WithdrawRequest;
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_schema::utils::{get_conn, DbPool};
use app_108jobs_db_schema_file::enums::TopUpStatus;
use app_108jobs_db_schema_file::enums::WithdrawStatus;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_db_views_wallet::api::{
  AdminTopUpWallet, AdminWalletOperationResponse, AdminWithdrawWallet, ListTopUpRequestQuery,
  ListTopUpRequestResponse, ListWithdrawRequestQuery, ListWithdrawRequestResponse,
  RejectWithdrawalRequest,
};
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};
use diesel_async::scoped_futures::ScopedFutureExt;

/// Deterministic idempotency key for the admin top-up flow. Stable across
/// retries so a duplicate request collides on the
/// `wallet_transaction(idempotency_key, wallet_id)` unique index instead of
/// generating a second credit. The qr_id is itself unique per top-up.
fn admin_top_up_idempotency_key(qr_id: &str) -> String {
  format!("admin:topup:qr:{qr_id}")
}

pub async fn admin_list_top_up_requests(
  query: Query<ListTopUpRequestQuery>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListTopUpRequestResponse>> {
  // Ensure admin access
  is_admin(&local_user_view)?;
  let res = list_top_up_requests_inner(&mut context.pool(), None, query.into_inner()).await?;

  Ok(Json(res))
}

pub async fn admin_top_up_wallet(
  data: Json<AdminTopUpWallet>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<AdminWalletOperationResponse>> {
  // Check if the user is admin
  is_admin(&local_user_view)?;

  // Resolve config (coin + platform escrow wallet) and translate the
  // target user id to a wallet id. These reads are not transactional with
  // the credit step — they only depend on data that does not change for
  // the duration of the call.
  let site_config = context.site_config().get().await?;
  let coin_id = site_config
    .site_view
    .local_site
    .coin_id
    .unwrap_or(CoinId(1));
  let platform_wallet_id = site_config
    .admins
    .first()
    .ok_or(FastJobErrorType::NoPlatformAdminConfigured)?
    .person
    .wallet_id;

  let target_wallet = WalletModel::get_by_user(&mut context.pool(), data.target_user_id).await?;

  admin_top_up_wallet_inner(
    &mut context.pool(),
    &data.qr_id,
    target_wallet.id,
    &data.reason,
    coin_id,
    platform_wallet_id,
    local_user_view.local_user.id,
  )
  .await
  .map(Json)
}

/// Atomic core of the admin top-up flow. Exposed to tests as a smaller
/// surface than the full Actix handler so the seam doesn't require a
/// FastJobContext / SCB client.
///
/// Invariants enforced in a single DB transaction:
///   1. The `top_up_requests` row is re-read with `SELECT ... FOR UPDATE`
///      so concurrent admin calls serialize on this row.
///   2. `transferred == false` and `status == Success` are checked under
///      the lock; a second concurrent call sees `transferred = true` and
///      returns `InvalidField("...already processed")` without crediting.
///   3. The wallet credit is journaled with a deterministic
///      `idempotency_key` derived from `qr_id`. If a retry somehow
///      bypasses the `transferred` flag (e.g. lost update from an older
///      code path), the
///      `wallet_transaction(idempotency_key, wallet_id)` unique index
///      rolls the whole txn back instead of producing a second credit.
///   4. `transferred` is flipped to `true` on the SAME connection inside
///      the txn, so the flag and the credit commit together or not at
///      all.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn admin_top_up_wallet_inner(
  pool: &mut DbPool<'_>,
  qr_id: &str,
  target_wallet_id: WalletId,
  reason: &str,
  coin_id: CoinId,
  platform_wallet_id: WalletId,
  admin_user_id: LocalUserId,
) -> FastJobResult<AdminWalletOperationResponse> {
  let conn = &mut get_conn(pool).await?;
  let idempotency_key = admin_top_up_idempotency_key(qr_id);

  let (new_balance, amount_coin) = conn
    .run_transaction(|tx| {
      async move {
        // 1) Re-read the request with FOR UPDATE so concurrent callers
        //    serialize and the transferred check is racing-safe.
        let locked = TopUpRequest::lock_for_credit_on_conn(tx, qr_id).await?;

        if locked.transferred {
          return Err::<
            (
              app_108jobs_db_schema::newtypes::Coin,
              app_108jobs_db_schema::newtypes::Coin,
            ),
            app_108jobs_utils::error::FastJobError,
          >(
            FastJobErrorType::InvalidField(
              "This top-up request has already been processed".to_string(),
            )
            .into(),
          );
        }
        if locked.status != TopUpStatus::Success {
          return Err(
            FastJobErrorType::InvalidField("Top-up request is not in Success status".to_string())
              .into(),
          );
        }

        let amount_coin = locked.amount_coin;

        // 2) Credit the target wallet via the connection-scoped helper so
        //    everything stays inside this txn. The deterministic
        //    idempotency_key collides on retries via the
        //    `wallet_transaction(idempotency_key, wallet_id)` unique index.
        let form = WalletTransactionInsertForm {
          wallet_id: target_wallet_id,
          reference_type: "admin_top_up".to_string(),
          reference_id: locked.id.0,
          kind: TxKind::Deposit,
          amount: amount_coin,
          description: reason.to_string(),
          counter_user_id: Some(admin_user_id),
          idempotency_key: idempotency_key.clone(),
        };
        WalletModel::deposit_from_platform_on_conn(tx, &form, coin_id, platform_wallet_id).await?;

        // 3) Flip transferred = true on the same connection.
        TopUpRequest::mark_transferred_on_conn(tx, qr_id).await?;

        // 4) Read the now-credited wallet's new balance for the response.
        let updated = WalletModel::read_by_id_on_conn(tx, target_wallet_id).await?;
        Ok((updated.balance_total, amount_coin))
      }
      .scope_boxed()
    })
    .await?;

  Ok(AdminWalletOperationResponse {
    wallet_id: target_wallet_id,
    new_balance,
    operation_amount: amount_coin,
    reason: reason.to_string(),
    success: true,
  })
}

/// Deterministic idempotency key for the admin withdraw flow. Stable across
/// retries so a duplicate approval collides on the
/// `wallet_transaction(idempotency_key, wallet_id)` unique index instead of
/// generating a second debit. `withdrawal_id` is a SERIAL primary key.
fn admin_withdraw_idempotency_key(withdrawal_id: WithdrawRequestId) -> String {
  format!("admin:withdraw:id:{}", withdrawal_id.0)
}

pub async fn admin_withdraw_wallet(
  data: Json<AdminWithdrawWallet>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<AdminWalletOperationResponse>> {
  // Check if user is admin
  is_admin(&local_user_view)?;

  // Resolve config + target wallet outside the transaction. None of these
  // values change during the call; only the wallet credit + status flip
  // need to be atomic.
  let site_config = context.site_config().get().await?;
  let coin_id = site_config
    .site_view
    .local_site
    .coin_id
    .unwrap_or(CoinId(1));
  let platform_wallet_id = site_config
    .admins
    .first()
    .ok_or(FastJobErrorType::NoPlatformAdminConfigured)?
    .person
    .wallet_id;

  let target_wallet = WalletModel::get_by_user(&mut context.pool(), data.target_user_id).await?;

  admin_withdraw_wallet_inner(
    &mut context.pool(),
    data.withdrawal_id,
    target_wallet.id,
    data.amount,
    &data.reason,
    coin_id,
    platform_wallet_id,
    local_user_view.local_user.id,
  )
  .await
  .map(Json)
}

/// Atomic core of the admin-approve-withdraw flow. Exposed to tests as a
/// smaller surface than the full Actix handler so the seam doesn't require
/// a FastJobContext.
///
/// Invariants enforced in a single DB transaction:
///   1. The `withdraw_requests` row is re-read with `SELECT ... FOR UPDATE`
///      so concurrent admin approvals serialize on this row.
///   2. `status == Pending` is checked under the lock; an already-Completed
///      or already-Rejected withdrawal returns `InvalidField` without
///      debiting the user wallet a second time.
///   3. The wallet debit is journaled with a deterministic
///      `idempotency_key` derived from `withdrawal_id`. If a retry bypasses
///      the status flag (e.g. lost update from an older code path), the
///      `wallet_transaction(idempotency_key, wallet_id)` unique index rolls
///      the whole txn back instead of producing a second debit.
///   4. `status = Completed` is set on the SAME connection inside the txn,
///      so the wallet movement and the status flip commit together or not
///      at all.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn admin_withdraw_wallet_inner(
  pool: &mut DbPool<'_>,
  withdrawal_id: WithdrawRequestId,
  target_wallet_id: WalletId,
  request_amount: Coin,
  reason: &str,
  coin_id: CoinId,
  platform_wallet_id: WalletId,
  admin_user_id: LocalUserId,
) -> FastJobResult<AdminWalletOperationResponse> {
  let conn = &mut get_conn(pool).await?;
  let idempotency_key = admin_withdraw_idempotency_key(withdrawal_id);

  let (new_balance, debit_amount) = conn
    .run_transaction(|tx| {
      async move {
        // 1) Re-read the request with FOR UPDATE so concurrent callers
        //    serialize on this row.
        let locked = WithdrawRequest::lock_for_approval_on_conn(tx, withdrawal_id).await?;

        // 2) Status guard. Only Pending withdrawals can be approved.
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

        // 3) The amount paid is the request's recorded amount; the
        //    `data.amount` field on the API is an admin-supplied value
        //    that historically didn't match the request. We trust the
        //    request row (locked under FOR UPDATE) as the source of
        //    truth. The handler's `request_amount` is preserved in the
        //    response for parity with the previous API shape.
        let _ = request_amount;
        let amount = locked.amount;

        // 4) Debit the user wallet to the platform escrow via the
        //    connection-scoped helper so everything stays inside this
        //    txn. The deterministic idempotency_key collides on retries.
        let form = WalletTransactionInsertForm {
          wallet_id: target_wallet_id,
          reference_type: "admin_withdraw".to_string(),
          reference_id: locked.id.0,
          kind: TxKind::Withdraw,
          amount,
          description: reason.to_string(),
          counter_user_id: Some(admin_user_id),
          idempotency_key: idempotency_key.clone(),
        };
        WalletModel::withdraw_to_platform_on_conn(tx, &form, coin_id, platform_wallet_id).await?;

        // 5) Flip status to Completed on the same connection.
        WithdrawRequest::set_status_on_conn(
          tx,
          withdrawal_id,
          WithdrawStatus::Completed,
          Some(reason.to_string()),
        )
        .await?;

        // 6) Read the now-debited wallet's new balance for the response.
        let updated = WalletModel::read_by_id_on_conn(tx, target_wallet_id).await?;
        Ok((updated.balance_total, amount))
      }
      .scope_boxed()
    })
    .await?;

  Ok(AdminWalletOperationResponse {
    wallet_id: target_wallet_id,
    new_balance,
    operation_amount: -debit_amount,
    reason: reason.to_string(),
    success: true,
  })
}

pub async fn admin_list_withdraw_requests(
  query: Query<ListWithdrawRequestQuery>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListWithdrawRequestResponse>> {
  is_admin(&local_user_view)?;
  let res = list_withdraw_requests_inner(&mut context.pool(), None, query.into_inner()).await?;

  Ok(Json(res))
}

/// Reject a pending withdraw request.
///
/// Atomic re-read with FOR UPDATE guards against the previously-possible
/// case where this handler could overwrite an already-Completed withdrawal
/// back to Rejected after the admin-approve handler had already paid it.
/// Only `Pending` rows can be rejected; re-rejecting an already-Rejected
/// row is a no-op (idempotent re-call).
pub async fn admin_reject_withdraw_request(
  data: Json<RejectWithdrawalRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  is_admin(&local_user_view)?;
  admin_reject_withdraw_request_inner(&mut context.pool(), data.withdrawal_id, &data.reason)
    .await?;
  Ok(Json(SuccessResponse::default()))
}

pub(crate) async fn admin_reject_withdraw_request_inner(
  pool: &mut DbPool<'_>,
  withdrawal_id: WithdrawRequestId,
  reason: &str,
) -> FastJobResult<()> {
  let conn = &mut get_conn(pool).await?;
  conn
    .run_transaction(|tx| {
      async move {
        let locked = WithdrawRequest::lock_for_approval_on_conn(tx, withdrawal_id).await?;
        match locked.status {
          WithdrawStatus::Pending => {
            WithdrawRequest::set_status_on_conn(
              tx,
              withdrawal_id,
              WithdrawStatus::Rejected,
              Some(reason.to_string()),
            )
            .await?;
            Ok::<(), app_108jobs_utils::error::FastJobError>(())
          }
          WithdrawStatus::Rejected => {
            // Idempotent re-call: already Rejected, leave it alone.
            Ok(())
          }
          WithdrawStatus::Completed => Err(
            FastJobErrorType::InvalidField(
              "This withdraw request has already been processed".to_string(),
            )
            .into(),
          ),
        }
      }
      .scope_boxed()
    })
    .await
}

// ============================================================================
// Integration test for admin_top_up_wallet_inner — exercises the atomic,
// deterministic-idempotency-key flow without spinning up an Actix app or a
// real FastJobContext.
//
// What it asserts:
//   1. A TopUpRequest with status=Success, transferred=false credits the
//      target wallet exactly once.
//   2. A second invocation with the same qr_id returns
//      InvalidField("...already been processed") and does NOT produce a
//      second wallet_transaction row.
//   3. The TopUpRequest.transferred flag is true after the first call.
//
// Before the fix at the top of this file, idempotency_key was
// `Uuid::new_v4()`, so the `wallet_transaction(idempotency_key, wallet_id)`
// unique index never fired on retry and the transferred check raced. This
// test fails on that code path; it passes against the atomic, deterministic
// implementation.
// ============================================================================
#[cfg(test)]
mod tests {
  use super::*;
  use app_108jobs_db_schema::newtypes::{Coin, LocalUserId as LUID};
  use app_108jobs_db_schema::source::coin::CoinModel;
  use app_108jobs_db_schema::source::currency::Currency;
  use app_108jobs_db_schema::source::instance::Instance;
  use app_108jobs_db_schema::source::local_user::{LocalUser, LocalUserInsertForm};
  use app_108jobs_db_schema::source::person::{Person, PersonInsertForm};
  use app_108jobs_db_schema::source::top_up_request::TopUpRequestInsertForm;
  use app_108jobs_db_schema::test_data::pool_for_tests;
  use app_108jobs_db_schema_file::schema::wallet_transaction;
  use chrono::Duration;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::RunQueryDsl;
  use serial_test::serial;

  struct Ctx {
    instance_id: app_108jobs_db_schema::newtypes::InstanceId,
    admin_user_id: LUID,
    target_user_id: LUID,
    target_wallet_id: WalletId,
    platform_wallet_id: WalletId,
    coin_id: CoinId,
    qr_id: String,
    amount_coin: Coin,
  }

  /// Build a self-contained fixture: instance + admin + target user + their
  /// wallets + a TopUpRequest in Success/non-transferred state.
  async fn fixture(pool: &mut DbPool<'_>) -> Ctx {
    let inst = Instance::read_or_create(
      pool,
      format!("admin-topup-test-{}.tld", uuid::Uuid::new_v4()),
    )
    .await
    .expect("create instance");

    let coin = CoinModel::ensure_platform_coin(pool)
      .await
      .expect("ensure platform coin");

    // Admin user with wallet. The admin's *personal* wallet doubles as the
    // platform escrow wallet in this codebase — production behavior at
    // `site_config.admins.first().person.wallet_id`.
    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let suffix_short = &suffix[..8];
    let (admin_form, admin_wallet) =
      PersonInsertForm::test_form_with_wallet(pool, inst.id, &format!("adm-{suffix_short}"))
        .await
        .expect("admin form");
    let admin_person = Person::create(pool, &admin_form)
      .await
      .expect("create admin person");
    let admin_local = LocalUser::create(
      pool,
      &LocalUserInsertForm::test_form_admin(admin_person.id),
      vec![],
    )
    .await
    .expect("create admin local_user");

    // Target user with wallet.
    let (tgt_form, tgt_wallet) =
      PersonInsertForm::test_form_with_wallet(pool, inst.id, &format!("tgt-{suffix_short}"))
        .await
        .expect("target form");
    let tgt_person = Person::create(pool, &tgt_form)
      .await
      .expect("create target person");
    let tgt_local = LocalUser::create(pool, &LocalUserInsertForm::test_form(tgt_person.id), vec![])
      .await
      .expect("create target local_user");

    // Currency (THB seeded by migration).
    let currency = Currency::get_default(pool)
      .await
      .expect("get default currency")
      .expect("THB seeded");

    // TopUpRequest in Success, not yet transferred.
    let qr_id = format!("qr-test-{}", uuid::Uuid::new_v4());
    let amount_coin = Coin(7_500);
    let form = TopUpRequestInsertForm {
      local_user_id: tgt_local.id,
      amount: 75.0,
      currency_id: currency.id,
      amount_coin,
      conversion_rate_used: 1,
      qr_id: qr_id.clone(),
      cs_ext_expiry_time: chrono::Utc::now() + Duration::minutes(5),
      paid_at: None,
    };
    let _created = TopUpRequest::create(pool, &form)
      .await
      .expect("insert top up request");
    // Mark it Success via the normal update path (mirrors what
    // `inquire_qrcode` does in production).
    let _ = TopUpRequest::update_by_qr_id(
      pool,
      qr_id.clone(),
      &app_108jobs_db_schema::source::top_up_request::TopUpRequestUpdateForm {
        status: Some(TopUpStatus::Success),
        updated_at: Some(chrono::Utc::now()),
        paid_at: Some(Some(chrono::Utc::now())),
        transferred: None,
      },
    )
    .await
    .expect("flip to Success");

    Ctx {
      instance_id: inst.id,
      admin_user_id: admin_local.id,
      target_user_id: tgt_local.id,
      target_wallet_id: tgt_wallet.id,
      platform_wallet_id: admin_wallet.id,
      coin_id: coin.id,
      qr_id,
      amount_coin,
      // suppress unused warning when test data evolves
      // (no field omitted intentionally — fixture is fully consumed)
    }
  }

  async fn cleanup(
    pool: &mut DbPool<'_>,
    instance_id: app_108jobs_db_schema::newtypes::InstanceId,
  ) {
    let _ = Instance::delete(pool, instance_id).await;
  }

  async fn count_admin_topup_user_credits(
    pool: &mut DbPool<'_>,
    wallet_id: WalletId,
    qr_id: &str,
  ) -> i64 {
    let key = admin_top_up_idempotency_key(qr_id);
    let conn = &mut get_conn(pool).await.expect("conn");
    wallet_transaction::table
      .filter(wallet_transaction::wallet_id.eq(wallet_id))
      .filter(wallet_transaction::idempotency_key.eq(key))
      .count()
      .get_result::<i64>(conn)
      .await
      .expect("count tx")
  }

  async fn read_wallet_balance(pool: &mut DbPool<'_>, wallet_id: WalletId) -> Coin {
    use app_108jobs_db_schema_file::schema::wallet;
    let conn = &mut get_conn(pool).await.expect("conn");
    wallet::table
      .find(wallet_id)
      .select(wallet::balance_total)
      .first::<Coin>(conn)
      .await
      .expect("balance_total")
  }

  async fn is_transferred(pool: &mut DbPool<'_>, qr_id: &str) -> bool {
    TopUpRequest::get_by_qr_id(pool, qr_id)
      .await
      .expect("re-read top up request")
      .transferred
  }

  /// Happy path: first call credits the user wallet exactly once and
  /// flips `transferred = true`. Second call with the same qr_id is
  /// rejected as already processed and produces no further credit.
  #[tokio::test]
  #[serial]
  async fn admin_top_up_wallet_inner_is_idempotent_on_retry() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = fixture(pool).await;

    let balance_before = read_wallet_balance(pool, ctx.target_wallet_id).await;

    // --- First call: must succeed ---
    let first = admin_top_up_wallet_inner(
      pool,
      &ctx.qr_id,
      ctx.target_wallet_id,
      "admin grants top-up",
      ctx.coin_id,
      ctx.platform_wallet_id,
      ctx.admin_user_id,
    )
    .await
    .expect("first admin_top_up must succeed");
    assert_eq!(first.operation_amount, ctx.amount_coin);
    assert!(first.success);

    let balance_after_first = read_wallet_balance(pool, ctx.target_wallet_id).await;
    assert_eq!(
      balance_after_first.0,
      balance_before.0 + ctx.amount_coin.0,
      "first call must credit exactly the topup amount"
    );
    assert!(
      is_transferred(pool, &ctx.qr_id).await,
      "transferred must be true after the first call"
    );
    assert_eq!(
      count_admin_topup_user_credits(pool, ctx.target_wallet_id, &ctx.qr_id).await,
      1,
      "exactly one wallet_transaction row for this qr_id"
    );

    // --- Second call: must fail as already processed, must NOT credit. ---
    let second = admin_top_up_wallet_inner(
      pool,
      &ctx.qr_id,
      ctx.target_wallet_id,
      "admin grants top-up",
      ctx.coin_id,
      ctx.platform_wallet_id,
      ctx.admin_user_id,
    )
    .await;
    let err = second.expect_err("second call must error");
    let msg = format!("{err:?}");
    assert!(
      msg.contains("already been processed") || msg.contains("InvalidField"),
      "second call must surface the already-processed guard, got: {msg}"
    );

    // Post-condition: balance unchanged from after the first call.
    let balance_after_second = read_wallet_balance(pool, ctx.target_wallet_id).await;
    assert_eq!(
      balance_after_second.0, balance_after_first.0,
      "second call must not move the balance"
    );
    // Post-condition: still exactly one journal entry under the
    // deterministic key.
    assert_eq!(
      count_admin_topup_user_credits(pool, ctx.target_wallet_id, &ctx.qr_id).await,
      1,
      "second call must not write another wallet_transaction"
    );

    // Silence unused warning on target_user_id — present for clarity in
    // the response (it is the entity the admin is crediting).
    let _ = ctx.target_user_id;
    cleanup(pool, ctx.instance_id).await;
  }

  // ==========================================================================
  // Integration test for admin_withdraw_wallet_inner — mirrors the top-up
  // test pattern. Builds a Pending WithdrawRequest, seeds the target wallet
  // with enough balance via WalletModel::deposit_from_platform, then calls
  // the inner twice and asserts the user wallet is debited exactly once.
  //
  // Before the matching fix in this file (random UUID + multi-pool-grab),
  // the second call would have:
  //   (a) blown past the WithdrawStatus check because the previous code
  //       did not check status at all, and
  //   (b) inserted a second wallet_transaction row because the random
  //       idempotency_key never collided on the
  //       `wallet_transaction(idempotency_key, wallet_id)` unique index.
  // ==========================================================================
  use app_108jobs_db_schema::newtypes::{BankAccountId, BankId, WithdrawRequestId};
  use app_108jobs_db_schema::source::bank::BankInsertForm;
  use app_108jobs_db_schema::source::user_bank_account::{BankAccount, UserBankAccountInsertForm};
  use app_108jobs_db_schema::source::withdraw_request::WithdrawRequestInsertForm;
  use app_108jobs_db_schema_file::schema::{banks, withdraw_requests};

  struct WithdrawCtx {
    instance_id: app_108jobs_db_schema::newtypes::InstanceId,
    admin_user_id: LUID,
    target_user_id: LUID,
    target_wallet_id: WalletId,
    platform_wallet_id: WalletId,
    coin_id: CoinId,
    withdrawal_id: WithdrawRequestId,
    amount_coin: Coin,
  }

  /// Build a Pending withdraw-request fixture with the user wallet
  /// pre-funded so the debit can actually succeed.
  async fn withdraw_fixture(pool: &mut DbPool<'_>) -> WithdrawCtx {
    let inst =
      Instance::read_or_create(pool, format!("admin-wd-test-{}.tld", uuid::Uuid::new_v4()))
        .await
        .expect("create instance");

    let coin = CoinModel::ensure_platform_coin(pool)
      .await
      .expect("ensure platform coin");

    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let suffix_short = &suffix[..8];

    // Admin user (their wallet doubles as platform escrow wallet in prod).
    let (admin_form, admin_wallet) =
      PersonInsertForm::test_form_with_wallet(pool, inst.id, &format!("adm-{suffix_short}"))
        .await
        .expect("admin form");
    let admin_person = Person::create(pool, &admin_form)
      .await
      .expect("create admin person");
    let admin_local = LocalUser::create(
      pool,
      &LocalUserInsertForm::test_form_admin(admin_person.id),
      vec![],
    )
    .await
    .expect("create admin local_user");

    // Target user with wallet.
    let (tgt_form, tgt_wallet) =
      PersonInsertForm::test_form_with_wallet(pool, inst.id, &format!("tgt-{suffix_short}"))
        .await
        .expect("target form");
    let tgt_person = Person::create(pool, &tgt_form)
      .await
      .expect("create target person");
    let tgt_local = LocalUser::create(pool, &LocalUserInsertForm::test_form(tgt_person.id), vec![])
      .await
      .expect("create target local_user");

    // Currency (THB seeded by migration).
    let currency = Currency::get_default(pool)
      .await
      .expect("get default currency")
      .expect("THB seeded");

    // Pre-fund the user wallet via the normal API so the debit has
    // something to draw from.
    let seed_amount = Coin(20_000);
    let seed_form = WalletTransactionInsertForm {
      wallet_id: tgt_wallet.id,
      reference_type: "test:seed".to_string(),
      reference_id: 0,
      kind: TxKind::Deposit,
      amount: seed_amount,
      description: "seed funds".to_string(),
      counter_user_id: Some(admin_local.id),
      idempotency_key: format!("test-seed-wd:{}", uuid::Uuid::new_v4()),
    };
    let _ = WalletModel::deposit_from_platform(pool, &seed_form, coin.id, admin_wallet.id)
      .await
      .expect("seed deposit");

    // Bank + bank-account for the target user (FK requirement on
    // withdraw_requests).
    let bank_id: BankId = {
      let conn = &mut get_conn(pool).await.expect("conn");
      let bank_form = BankInsertForm {
        name: format!("Test Bank {suffix_short}"),
        country_id: "TH".to_string(),
        bank_code: Some(format!("WB{suffix_short}")),
        swift_code: None,
        is_active: Some(true),
      };
      let id: i32 = diesel::insert_into(banks::table)
        .values(&bank_form)
        .returning(banks::id)
        .get_result(conn)
        .await
        .expect("insert bank");
      BankId(id)
    };
    let bank_account: BankAccount = BankAccount::create(
      pool,
      &UserBankAccountInsertForm {
        local_user_id: tgt_local.id,
        bank_id,
        account_number: "1234567890".to_string(),
        account_name: format!("Holder {suffix_short}"),
        verification_image_path: None,
      },
    )
    .await
    .expect("create bank account");

    // Pending WithdrawRequest for half the seeded balance.
    let amount = Coin(5_000);
    let withdrawal_id: WithdrawRequestId = {
      let conn = &mut get_conn(pool).await.expect("conn");
      let form = WithdrawRequestInsertForm {
        local_user_id: tgt_local.id,
        wallet_id: tgt_wallet.id,
        user_bank_account_id: bank_account.id,
        amount,
        currency_id: currency.id,
        amount_currency: amount.0 as f64,
        conversion_rate_used: 1,
        reason: Some("test withdrawal".to_string()),
      };
      let id: WithdrawRequestId = diesel::insert_into(withdraw_requests::table)
        .values(&form)
        .returning(withdraw_requests::id)
        .get_result(conn)
        .await
        .expect("insert withdraw_request");
      id
    };

    let _ = (bank_account.id as BankAccountId,); // silence unused

    WithdrawCtx {
      instance_id: inst.id,
      admin_user_id: admin_local.id,
      target_user_id: tgt_local.id,
      target_wallet_id: tgt_wallet.id,
      platform_wallet_id: admin_wallet.id,
      coin_id: coin.id,
      withdrawal_id,
      amount_coin: amount,
    }
  }

  async fn count_admin_withdraw_user_debits(
    pool: &mut DbPool<'_>,
    wallet_id: WalletId,
    withdrawal_id: WithdrawRequestId,
  ) -> i64 {
    let key = admin_withdraw_idempotency_key(withdrawal_id);
    let conn = &mut get_conn(pool).await.expect("conn");
    wallet_transaction::table
      .filter(wallet_transaction::wallet_id.eq(wallet_id))
      .filter(wallet_transaction::idempotency_key.eq(key))
      .count()
      .get_result::<i64>(conn)
      .await
      .expect("count tx")
  }

  async fn read_withdraw_status(pool: &mut DbPool<'_>, id: WithdrawRequestId) -> WithdrawStatus {
    let conn = &mut get_conn(pool).await.expect("conn");
    withdraw_requests::table
      .find(id)
      .select(withdraw_requests::status)
      .first::<WithdrawStatus>(conn)
      .await
      .expect("status")
  }

  /// Approve once → wallet debited exactly by the request amount, status flips
  /// Completed. Retry → InvalidField/already-processed, no further debit.
  #[tokio::test]
  #[serial]
  async fn admin_withdraw_wallet_inner_is_idempotent_on_retry() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = withdraw_fixture(pool).await;
    let balance_before = read_wallet_balance(pool, ctx.target_wallet_id).await;

    // --- First call: succeeds, debits exactly once. ---
    let first = admin_withdraw_wallet_inner(
      pool,
      ctx.withdrawal_id,
      ctx.target_wallet_id,
      ctx.amount_coin,
      "admin approves withdrawal",
      ctx.coin_id,
      ctx.platform_wallet_id,
      ctx.admin_user_id,
    )
    .await
    .expect("first admin_withdraw must succeed");
    assert_eq!(first.operation_amount, Coin(-ctx.amount_coin.0));
    assert!(first.success);

    let balance_after_first = read_wallet_balance(pool, ctx.target_wallet_id).await;
    assert_eq!(
      balance_after_first.0,
      balance_before.0 - ctx.amount_coin.0,
      "first call must debit exactly the requested amount"
    );
    assert_eq!(
      read_withdraw_status(pool, ctx.withdrawal_id).await,
      WithdrawStatus::Completed,
      "first call must flip status to Completed"
    );
    assert_eq!(
      count_admin_withdraw_user_debits(pool, ctx.target_wallet_id, ctx.withdrawal_id).await,
      1,
      "exactly one wallet_transaction row for this withdrawal_id"
    );

    // --- Second call: must fail as already processed; must NOT debit. ---
    let second = admin_withdraw_wallet_inner(
      pool,
      ctx.withdrawal_id,
      ctx.target_wallet_id,
      ctx.amount_coin,
      "admin approves withdrawal",
      ctx.coin_id,
      ctx.platform_wallet_id,
      ctx.admin_user_id,
    )
    .await;
    let err = second.expect_err("second call must error");
    let msg = format!("{err:?}");
    assert!(
      msg.contains("already been processed") || msg.contains("InvalidField"),
      "second call must surface the already-processed guard, got: {msg}"
    );

    // Post-conditions: no further movement.
    let balance_after_second = read_wallet_balance(pool, ctx.target_wallet_id).await;
    assert_eq!(
      balance_after_second.0, balance_after_first.0,
      "second call must not move the balance"
    );
    assert_eq!(
      count_admin_withdraw_user_debits(pool, ctx.target_wallet_id, ctx.withdrawal_id).await,
      1,
      "second call must not write another wallet_transaction"
    );

    let _ = ctx.target_user_id;
    cleanup(pool, ctx.instance_id).await;
  }

  /// A withdraw request that was previously rejected cannot be approved.
  /// Pre-fix code did not check status and would have debited the wallet.
  #[tokio::test]
  #[serial]
  async fn admin_withdraw_rejects_already_rejected_request() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = withdraw_fixture(pool).await;
    let balance_before = read_wallet_balance(pool, ctx.target_wallet_id).await;

    // Force the row into Rejected state directly.
    admin_reject_withdraw_request_inner(pool, ctx.withdrawal_id, "policy violation")
      .await
      .expect("reject");
    assert_eq!(
      read_withdraw_status(pool, ctx.withdrawal_id).await,
      WithdrawStatus::Rejected
    );

    // Approval must now refuse.
    let err = admin_withdraw_wallet_inner(
      pool,
      ctx.withdrawal_id,
      ctx.target_wallet_id,
      ctx.amount_coin,
      "post-reject approval attempt",
      ctx.coin_id,
      ctx.platform_wallet_id,
      ctx.admin_user_id,
    )
    .await
    .expect_err("must refuse rejected withdrawals");
    assert!(
      format!("{err:?}").contains("rejected") || format!("{err:?}").contains("InvalidField"),
      "expected rejected-not-approvable error, got {err:?}"
    );

    // Balance unchanged; no journal entry.
    assert_eq!(
      read_wallet_balance(pool, ctx.target_wallet_id).await.0,
      balance_before.0
    );
    assert_eq!(
      count_admin_withdraw_user_debits(pool, ctx.target_wallet_id, ctx.withdrawal_id).await,
      0,
    );
    cleanup(pool, ctx.instance_id).await;
  }

  /// Rejecting an already-Completed withdrawal must refuse — prevents the
  /// previously-possible silent status overwrite from Completed -> Rejected
  /// after funds have already moved.
  #[tokio::test]
  #[serial]
  async fn admin_reject_withdraw_refuses_already_completed_request() {
    let pool = pool_for_tests();
    let pool = &mut (&pool).into();
    let ctx = withdraw_fixture(pool).await;

    admin_withdraw_wallet_inner(
      pool,
      ctx.withdrawal_id,
      ctx.target_wallet_id,
      ctx.amount_coin,
      "approve first",
      ctx.coin_id,
      ctx.platform_wallet_id,
      ctx.admin_user_id,
    )
    .await
    .expect("approve");
    assert_eq!(
      read_withdraw_status(pool, ctx.withdrawal_id).await,
      WithdrawStatus::Completed
    );

    let err = admin_reject_withdraw_request_inner(pool, ctx.withdrawal_id, "late reject")
      .await
      .expect_err("must refuse rejecting a completed withdrawal");
    assert!(
      format!("{err:?}").contains("already been processed")
        || format!("{err:?}").contains("InvalidField"),
      "expected already-processed error, got {err:?}"
    );

    // Status must remain Completed.
    assert_eq!(
      read_withdraw_status(pool, ctx.withdrawal_id).await,
      WithdrawStatus::Completed
    );
    cleanup(pool, ctx.instance_id).await;
  }
}
