#[cfg(feature = "full")]
use diesel::{Queryable, Selectable};
use lemmy_db_schema::source::{
  chat_message::ChatMessage, chat_participant::ChatParticipant, chat_room::ChatRoom,
  local_user::LocalUser, post::Post, comment::Comment,
};
use serde::{Deserialize, Serialize};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// A chat message view, including sender and room.
pub struct ChatMessageView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub message: ChatMessage,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub sender: LocalUser,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub room: ChatRoom,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// A chat room view, including its participants. Not a Diesel selectable struct; constructed manually.
pub struct ChatRoomView {
  pub room: ChatRoom,
  // Not selectable by Diesel; assembled separately via additional query
  pub participants: Vec<ChatParticipant>,
  pub post: Option<Post>,
  pub current_comment: Option<Comment>,
}
