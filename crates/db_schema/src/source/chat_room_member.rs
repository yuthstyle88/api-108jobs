use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::chat_room_member;
use crate::newtypes::{ChatRoomId, LocalUserId};

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = chat_room_member))]
#[cfg_attr(feature = "full", diesel(primary_key(room_id, user_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct ChatRoomMember {
    pub room_id: ChatRoomId,
    pub user_id: LocalUserId,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(
    feature = "full",
    derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = chat_room_member))]
pub struct ChatRoomMemberInsertForm {
    pub room_id: ChatRoomId,
    pub user_id: LocalUserId,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(Serialize, Deserialize, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = chat_room_member))]
pub struct ChatRoomMemberUpdateForm {
    pub room_id: ChatRoomId,
    pub user_id: LocalUserId,
}