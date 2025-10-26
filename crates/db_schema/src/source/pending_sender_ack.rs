use crate::newtypes::{ChatRoomId, LocalUserId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::pending_sender_ack;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = pending_sender_ack))]
#[cfg_attr(feature = "full", diesel(primary_key(id)))]
#[serde(rename_all = "camelCase")]
pub struct PendingSenderAck {
  pub id: i64,
  pub room_id: ChatRoomId,
  pub sender_id: LocalUserId,
  pub client_id: uuid::Uuid,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(
  feature = "full",
  derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = pending_sender_ack))]
pub struct PendingSenderAckInsertForm {
  pub room_id: ChatRoomId,
  pub sender_id: Option<LocalUserId>,
  pub client_id: Option<uuid::Uuid>,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(Serialize, Deserialize, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = pending_sender_ack))]
pub struct PendingSenderAckUpdateForm{
  pub sender_id: Option<LocalUserId>,
}
