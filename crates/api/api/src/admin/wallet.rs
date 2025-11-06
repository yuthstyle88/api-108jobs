use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::is_admin;
use lemmy_db_schema::newtypes::CoinId;
use lemmy_db_schema::source::wallet::{TxKind, WalletModel, WalletTransactionInsertForm};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_wallet::api::{
  AdminTopUpWallet, AdminWalletOperationResponse, AdminWithdrawWallet, ListWalletTopupsQuery,
  ListWalletTopupsResponse,
};
use lemmy_db_views_wallet::WalletTopupView;
use lemmy_utils::error::FastJobResult;
use uuid::Uuid;

pub async fn admin_list_wallet_topups(
  query: Query<ListWalletTopupsQuery>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListWalletTopupsResponse>> {
  let data = query.into_inner();
  // Ensure admin access
  is_admin(&local_user_view)?;
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(WalletTopupView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };
  let items = WalletTopupView::list(&mut context.pool(), None, cursor_data, data).await?;
  let next_page = items.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = items.first().map(PaginationCursorBuilder::to_cursor);
  Ok(Json(ListWalletTopupsResponse {
    wallet_topups: items,
    next_page,
    prev_page,
  }))
}

pub async fn admin_top_up_wallet(
  data: Json<AdminTopUpWallet>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<AdminWalletOperationResponse>> {
  // Check if the user is admin
  is_admin(&local_user_view)?;
  let site_config = context.site_config().get().await?;
  let coin_id = site_config
    .site_view
    .local_site
    .coin_id
    .unwrap_or(CoinId(1));
  let platform_wallet_id = site_config
    .admins
    .first()
    .expect("At least one admin must exist to perform admin wallet operations")
    .person
    .wallet_id;

  let form = WalletTransactionInsertForm {
    wallet_id: data.wallet_id,
    reference_type: "admin_top_up".to_string(),
    reference_id: 0,
    kind: TxKind::Deposit,
    amount: data.amount,
    description: data.reason.clone(),
    counter_user_id: None,
    idempotency_key: Uuid::new_v4().to_string(),
  };

  let wallet =
    WalletModel::create_transaction(&mut context.pool(), &form, coin_id, platform_wallet_id)
      .await?;

  Ok(Json(AdminWalletOperationResponse {
    wallet_id: data.wallet_id,
    new_balance: wallet.balance_total,
    operation_amount: data.amount,
    reason: data.reason.clone(),
    success: true,
  }))
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
    .first()
    .expect("At least one admin must exist to perform admin wallet operations")
    .person
    .wallet_id;

  let form = WalletTransactionInsertForm {
    wallet_id: data.wallet_id,
    reference_type: "admin_withdraw".to_string(),
    reference_id: 0,
    kind: TxKind::Withdraw,
    amount: data.amount,
    description: data.reason.clone(),
    counter_user_id: None,
    idempotency_key: Uuid::new_v4().to_string(),
  };

  let wallet =
    WalletModel::create_transaction(&mut context.pool(), &form, coin_id, platform_wallet_id)
      .await?;

  Ok(Json(AdminWalletOperationResponse {
    wallet_id: data.wallet_id,
    new_balance: wallet.balance_total,
    operation_amount: -data.amount, // Negative because it's a withdrawal
    reason: data.reason.clone(),
    success: true,
  }))
}
