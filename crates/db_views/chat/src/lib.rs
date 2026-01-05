use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel::{Queryable, Selectable};
use app_108jobs_db_schema::newtypes::{ChatRoomId, DbUrl, LocalUserId, PersonId};
use app_108jobs_db_schema::source::workflow::Workflow;
use app_108jobs_db_schema::source::{
  chat_message::ChatMessage, chat_room::ChatRoom, local_user::LocalUser,
};
use serde::{Deserialize, Serialize};
use app_108jobs_db_views_post::PostPreview;

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub message: ChatMessage,
  #[serde(skip_serializing)]
  #[cfg_attr(feature = "full", diesel(embed))]
  pub sender: LocalUser,
  #[serde(skip_serializing)]
  #[cfg_attr(feature = "full", diesel(embed))]
  pub room: ChatRoom,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatRoomView {
  pub room: ChatRoom,
  pub participants: Vec<ChatParticipantView>,
  pub post: Option<PostPreview>,
  pub last_message: Option<ChatMessage>,
  pub workflow: Option<Workflow>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[cfg_attr(feature = "full", derive(Queryable))]
#[serde(rename_all = "camelCase")]
pub struct ChatParticipantView {
  pub id: LocalUserId,
  pub person_id: PersonId,
  pub name: String,
  pub display_name: Option<String>,
  pub avatar: Option<DbUrl>,
  pub available: bool,
  pub room_id: ChatRoomId,
  pub joined_at: DateTime<Utc>,
}
