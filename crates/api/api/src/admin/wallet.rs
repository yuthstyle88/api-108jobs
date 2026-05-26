use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_api_utils::utils::{
  is_admin, list_top_up_requests_inner, list_withdraw_requests_inner,
};
use app_108jobs_db_schema::newtypes::{CoinId, LocalUserId, WalletId};
use app_108jobs_db_schema::source::top_up_request::TopUpRequest;
use app_108jobs_db_schema::source::wallet::{TxKind, WalletModel, WalletTransactionInsertForm};
use app_108jobs_db_schema::source::withdraw_request::{WithdrawRequest, WithdrawRequestUpdateForm};
use app_108jobs_db_schema::traits::Crud;
use app_108jobs_db_schema::utils::{get_conn, DbPool};
use app_108jobs_db_schema_file::enums::TopUpStatus;
use app_108jobs_db_schema_file::enums::WithdrawStatus;
use app_108jobs_db_schema_file::schema::top_up_requests;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_db_views_wallet::api::{
  AdminTopUpWallet, AdminWalletOperationResponse, AdminWithdrawWallet, ListTopUpRequestQuery,
  ListTopUpRequestResponse, ListWithdrawRequestQuery, ListWithdrawRequestResponse,
  RejectWithdrawalRequest,
};
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{scoped_futures::ScopedFutureExt, RunQueryDsl};
use uuid::Uuid;

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

  let (wallet_new_balance, amount_coin) = conn
    .run_transaction(|tx| {
      async move {
        // 1) Re-read the request with FOR UPDATE so concurrent callers
        //    serialize and the transferred check is racing-safe.
        let locked = top_up_requests::table
          .filter(top_up_requests::qr_id.eq(qr_id))
          .for_update()
          .first::<TopUpRequest>(tx)
          .await
          .with_fastjob_type(FastJobErrorType::NotFound)?;

        if locked.transferred {
          return Err::<(app_108jobs_db_schema::source::wallet::Wallet, app_108jobs_db_schema::newtypes::Coin), app_108jobs_utils::error::FastJobError>(
            FastJobErrorType::InvalidField(
              "This top-up request has already been processed".to_string(),
            )
            .into(),
          );
        }
        if locked.status != TopUpStatus::Success {
          return Err(
            FastJobErrorType::InvalidField(
              "Top-up request is not in Success status".to_string(),
            )
            .into(),
          );
        }

        let amount_coin = locked.amount_coin;

        // 2) Credit the target wallet via the connection-scoped helper so
        //    everything stays inside this txn. The deterministic
        //    idempotency_key collides on retries.
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
        WalletModel::deposit_from_platform_on_conn(tx, &form, coin_id, platform_wallet_id)
          .await?;

        // 3) Flip transferred = true on the same connection.
        diesel::update(
          top_up_requests::table.filter(top_up_requests::qr_id.eq(qr_id)),
        )
        .set((
          top_up_requests::transferred.eq(true),
          top_up_requests::updated_at.eq(Utc::now()),
        ))
        .execute(tx)
        .await
        .with_fastjob_type(FastJobErrorType::DatabaseError)?;

        // 4) Re-read the now-credited wallet to return its new balance.
        use app_108jobs_db_schema_file::schema::wallet;
        let updated_wallet: app_108jobs_db_schema::source::wallet::Wallet = wallet::table
          .find(target_wallet_id)
          .first(tx)
          .await
          .with_fastjob_type(FastJobErrorType::CouldntFindWalletByUser)?;

        Ok((updated_wallet, amount_coin))
      }
      .scope_boxed()
    })
    .await
    .map(|(w, a)| (w.balance_total, a))?;

  Ok(AdminWalletOperationResponse {
    wallet_id: target_wallet_id,
    new_balance: wallet_new_balance,
    operation_amount: amount_coin,
    reason: reason.to_string(),
    success: true,
  })
}

pub async fn admin_withdraw_wallet(
  data: Json<AdminWithdrawWallet>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<AdminWalletOperationResponse>> {
  // Check if user is admin
  is_admin(&local_user_view)?;

  // Fetch site config once to avoid redundant await calls and clones
  let site_config = context.site_config().get().await?;
  let coin_id = site_config
    .site_view
    .local_site
    .coin_id
    .unwrap_or(CoinId(1));
  let platform_wallet_id = site_config
    .admins
    .get(0)
    .expect("At least one admin must exist to perform admin wallet operations")
    .person
    .wallet_id;

  let target_wallet = WalletModel::get_by_user(&mut context.pool(), data.target_user_id).await?;

  let form = WalletTransactionInsertForm {
    wallet_id: target_wallet.id,
    reference_type: "admin_withdraw".to_string(),
    reference_id: 0,
    kind: TxKind::Withdraw,
    amount: data.amount,
    description: data.reason.clone(),
    counter_user_id: Some(local_user_view.local_user.id),
    idempotency_key: Uuid::new_v4().to_string(),
  };

  let wallet =
    WalletModel::create_transaction(&mut context.pool(), &form, coin_id, platform_wallet_id)
      .await?;

  let withdrawal_update_form = WithdrawRequestUpdateForm {
    status: Some(WithdrawStatus::Completed),
    updated_at: Some(Utc::now()),
    reason: Some(Some(data.reason.clone())),
  };

  let _updated = WithdrawRequest::update(
    &mut context.pool(),
    data.withdrawal_id,
    &withdrawal_update_form,
  )
  .await?;

  Ok(Json(AdminWalletOperationResponse {
    wallet_id: wallet.id,
    new_balance: wallet.balance_total,
    operation_amount: -data.amount,
    reason: data.reason.clone(),
    success: true,
  }))
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

pub async fn admin_reject_withdraw_request(
  data: Json<RejectWithdrawalRequest>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  is_admin(&local_user_view)?;

  let update_form = WithdrawRequestUpdateForm {
    status: Some(WithdrawStatus::Rejected),
    updated_at: Some(Utc::now()),
    reason: Some(Some(data.reason.clone())),
  };

  let _updated =
    WithdrawRequest::update(&mut context.pool(), data.withdrawal_id, &update_form).await?;

  Ok(Json(SuccessResponse::default()))
}
