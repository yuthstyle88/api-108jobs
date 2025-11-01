#[cfg(feature = "full")]
use diesel::{Queryable, Selectable};
use lemmy_db_schema::source::{
  chat_message::ChatMessage, chat_participant::ChatParticipant, chat_room::ChatRoom,
  local_user::LocalUser, post::Post, comment::Comment,
};
use serde::{Deserialize, Serialize};
use lemmy_db_schema::source::person::Person;
use lemmy_db_schema::source::workflow::Workflow;

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
  #[serde(skip_serializing)]
  #[cfg_attr(feature = "full", diesel(embed))]
  pub sender: LocalUser,
  #[serde(skip_serializing)]
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
  pub participants: Vec<ChatParticipantView>,
  pub post: Option<Post>,
  pub current_comment: Option<Comment>,
  pub last_message: Option<ChatMessage>,
  pub workflow: Option<Workflow>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// A chat participant view. Not a Diesel selectable struct; constructed manually.
pub struct ChatParticipantView {
  pub participant: ChatParticipant,
  pub member_person: Person,
}