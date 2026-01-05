use crate::newtypes::{LocalUserId, TopUpRequestId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use app_108jobs_db_schema_file::enums::TopUpStatus;
#[cfg(feature = "full")]
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {i_love_jesus::CursorKeysModule, app_108jobs_db_schema_file::schema::top_up_requests};

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = top_up_requests))]
#[cfg_attr(feature = "full", diesel(primary_key(id)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = top_up_requests_keys))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[serde(rename_all = "camelCase")]
pub struct TopUpRequest {
  pub id: TopUpRequestId,
  pub local_user_id: LocalUserId,
  pub amount: f64,
  pub currency_name: String,
  pub qr_id: String,
  pub cs_ext_expiry_time: DateTime<Utc>,
  pub status: TopUpStatus,
  pub transferred: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub paid_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(
  feature = "full",
  derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = top_up_requests))]
pub struct TopUpRequestInsertForm {
  pub local_user_id: LocalUserId,
  pub amount: f64,
  pub currency_name: String,
  pub qr_id: String,
  pub cs_ext_expiry_time: DateTime<Utc>,
  pub paid_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(Serialize, Deserialize, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = top_up_requests))]
pub struct TopUpRequestUpdateForm {
  pub status: Option<TopUpStatus>,
  pub updated_at: Option<DateTime<Utc>>,
  pub paid_at: Option<Option<DateTime<Utc>>>,
  pub transferred: Option<bool>,
}
