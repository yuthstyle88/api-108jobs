use crate::newtypes::{ChatRoomId, LocalUserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
// Bring the generated diesel schema table into scope for annotations
use app_108jobs_db_schema_file::schema::chat_unread;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = chat_unread)]
#[diesel(primary_key(local_user_id, room_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[serde(rename_all = "camelCase")]
pub struct ChatUnread {
  pub local_user_id: LocalUserId,
  pub room_id: ChatRoomId,
  pub unread_count: i32,
  pub last_message_id: Option<String>,
  pub last_message_at: Option<DateTime<Utc>>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, derive_new::new, Serialize, Deserialize, diesel::Insertable)]
#[diesel(table_name = chat_unread)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatUnreadUpsertForm {
  pub local_user_id: LocalUserId,
  pub room_id: ChatRoomId,
  pub unread_count: i32,
  pub last_message_id: Option<String>,
  pub last_message_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatUnreadUpdateForm {
  pub unread_count: Option<i32>,
  pub last_message_id: Option<Option<String>>,
  pub last_message_at: Option<Option<DateTime<Utc>>>,
}
