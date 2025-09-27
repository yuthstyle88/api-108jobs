use crate::newtypes::{LocalUserId, ChatRoomId, ChatMessageId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::last_reads;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = last_reads))]
#[cfg_attr(feature = "full", diesel(primary_key(user_id, room_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct LastRead {
    pub user_id: LocalUserId,
    pub room_id: ChatRoomId,
    pub last_read_msg_id: ChatMessageId,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = last_reads))]
pub struct LastReadInsertForm {
    pub user_id: LocalUserId,
    pub room_id: ChatRoomId,
    pub last_read_msg_id: ChatMessageId,
    #[new(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = last_reads))]
pub struct LastReadUpdateForm {
    pub last_read_msg_id: Option<ChatMessageId>,
    pub updated_at: Option<DateTime<Utc>>,
}
