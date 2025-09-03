use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::chat_participant;
use crate::newtypes::{ChatRoomId, LocalUserId};

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = chat_participant))]
#[cfg_attr(feature = "full", diesel(primary_key(room_id, member_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[serde(rename_all = "camelCase")]
pub struct ChatParticipant {
    pub room_id: ChatRoomId,
    pub member_id: LocalUserId,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(
    feature = "full",
    derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = chat_participant))]
pub struct ChatParticipantInsertForm {
    pub room_id: ChatRoomId,
    pub member_id: LocalUserId,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(Serialize, Deserialize, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = chat_participant))]
pub struct ChatParticipantUpdateForm {
    pub room_id: ChatRoomId,
    pub member_id: LocalUserId,
}