//! Escrow hold ledger.
//!
//! See migration `2026-05-25-180000_add_wallet_versioning_and_hold_ledger`.
//!
//! Each row records a per-billing escrow allocation against a wallet. The
//! sum of `amount` across `status='Active'` rows for a wallet is the
//! authoritative source for "how much is held". `wallet.balance_outstanding`
//! is kept in lockstep for backward compatibility with existing read paths.
//!
//! A partial unique index `uq_wallet_hold_active_per_billing` enforces
//! "at most one active hold per billing" at the DB level — duplicate approve
//! attempts collide here and surface as `DuplicateWalletHold`.

use crate::newtypes::{BillingId, Coin, WalletHoldId, WalletId};
#[cfg(feature = "full")]
use app_108jobs_db_schema_file::schema::wallet_hold;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// String constants for the `status` column. Matches the CHECK constraint
/// in the up.sql migration. We use a Rust enum at the call-site but persist
/// as text so the migration stays reversible without enum-drop hazards.
pub mod hold_status {
  pub const ACTIVE: &str = "Active";
  pub const RELEASED: &str = "Released";
  pub const CAPTURED: &str = "Captured";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HoldStatus {
  Active,
  Released,
  Captured,
}

impl HoldStatus {
  pub fn as_str(&self) -> &'static str {
    match self {
      HoldStatus::Active => hold_status::ACTIVE,
      HoldStatus::Released => hold_status::RELEASED,
      HoldStatus::Captured => hold_status::CAPTURED,
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = wallet_hold))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct WalletHold {
  pub id: WalletHoldId,
  pub wallet_id: WalletId,
  pub billing_id: BillingId,
  pub amount: Coin,
  /// One of `Active` / `Released` / `Captured` — see `hold_status` module.
  pub status: String,
  pub idempotency_key: Option<String>,
  pub created_at: DateTime<Utc>,
  pub released_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = wallet_hold))]
pub struct WalletHoldInsertForm {
  pub wallet_id: WalletId,
  pub billing_id: BillingId,
  pub amount: Coin,
  pub status: String,
  pub idempotency_key: Option<String>,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = wallet_hold))]
pub struct WalletHoldUpdateForm {
  pub status: Option<String>,
  pub released_at: Option<Option<DateTime<Utc>>>,
}
