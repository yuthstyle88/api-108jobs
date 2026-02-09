//! Platform asset admin endpoints
//! Provides admin APIs to view platform wallet and coin status

use actix_web::web::{Data, Json};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::newtypes::Coin;
use app_108jobs_db_schema::source::coin::CoinModel;
use app_108jobs_db_schema::source::wallet::WalletModel;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_utils::error::FastJobResult;
use serde::{Deserialize, Serialize};

// ============================================================================
// Platform Admin API Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
/// Platform wallet balance details
pub struct PlatformWalletBalance {
  pub wallet_id: i32,
  pub balance_total: Coin,
  pub balance_available: Coin,
  pub balance_outstanding: Coin,
  pub is_negative: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
/// Platform coin details
pub struct PlatformCoinDetails {
  pub coin_id: i32,
  pub code: String,
  pub name: String,
  pub supply_total: Coin,
  pub supply_minted_total: Coin,
  pub supply_burned: Coin,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
/// Complete platform assets response
pub struct PlatformAssetsResponse {
  pub wallet: PlatformWalletBalance,
  pub coin: PlatformCoinDetails,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
/// Platform wallet balance response
pub struct PlatformBalanceResponse {
  pub wallet: PlatformWalletBalance,
}

// ============================================================================
// Platform Admin Endpoints
// ============================================================================

/// Get platform assets (wallet + coin)
/// Admin only endpoint for viewing platform asset status
pub async fn admin_get_platform_assets(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PlatformAssetsResponse>> {
  is_admin(&local_user_view)?;

  // Get platform wallet
  let wallet = WalletModel::get_platform_wallet(&mut context.pool()).await?;
  let wallet_balance = PlatformWalletBalance {
    wallet_id: wallet.id.0,
    balance_total: wallet.balance_total,
    balance_available: wallet.balance_available,
    balance_outstanding: wallet.balance_outstanding,
    is_negative: wallet.balance_total.0 < 0,
  };

  // Get platform coin
  let coin = CoinModel::get_platform_coin(&mut context.pool()).await?;
  let supply_burned = Coin(coin.supply_total.0 - coin.supply_minted_total.0);
  let coin_details = PlatformCoinDetails {
    coin_id: coin.id.0,
    code: coin.code,
    name: coin.name,
    supply_total: coin.supply_total,
    supply_minted_total: coin.supply_minted_total,
    supply_burned,
  };

  Ok(Json(PlatformAssetsResponse {
    wallet: wallet_balance,
    coin: coin_details,
  }))
}

/// Get platform wallet balance
/// Admin only endpoint for viewing platform wallet balance
pub async fn admin_get_platform_balance(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<PlatformBalanceResponse>> {
  is_admin(&local_user_view)?;

  // Get platform wallet
  let wallet = WalletModel::get_platform_wallet(&mut context.pool()).await?;
  let wallet_balance = PlatformWalletBalance {
    wallet_id: wallet.id.0,
    balance_total: wallet.balance_total,
    balance_available: wallet.balance_available,
    balance_outstanding: wallet.balance_outstanding,
    is_negative: wallet.balance_total.0 < 0,
  };

  Ok(Json(PlatformBalanceResponse {
    wallet: wallet_balance,
  }))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn is_admin(local_user_view: &LocalUserView) -> FastJobResult<()> {
  app_108jobs_api_utils::utils::is_admin(local_user_view)
}
