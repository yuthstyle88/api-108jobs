use crate::newtypes::{LocalUserId, WalletTopupId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::enums::TopupStatus;
#[cfg(feature = "full")]
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {i_love_jesus::CursorKeysModule, lemmy_db_schema_file::schema::wallet_topups};

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = wallet_topups))]
#[cfg_attr(feature = "full", diesel(primary_key(id)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = wallet_topups_keys))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[serde(rename_all = "camelCase")]
pub struct WalletTopup {
  pub id: WalletTopupId,
  pub local_user_id: LocalUserId,
  pub amount: f64,
  pub currency_name: String,
  pub qr_id: String,
  pub cs_ext_expiry_time: DateTime<Utc>,
  pub status: TopupStatus,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub paid_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(
  feature = "full",
  derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = wallet_topups))]
pub struct WalletTopupInsertForm {
  pub local_user_id: LocalUserId,
  pub amount: f64,
  pub currency_name: String,
  pub qr_id: String,
  pub cs_ext_expiry_time: DateTime<Utc>,
  pub paid_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(Serialize, Deserialize, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = wallet_topups))]
pub struct WalletTopupUpdateForm {
  pub status: Option<TopupStatus>,
  pub updated_at: Option<DateTime<Utc>>,
  pub paid_at: Option<Option<DateTime<Utc>>>,
}
