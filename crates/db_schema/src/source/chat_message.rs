use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::chat_message;
use crate::newtypes::{ChatRoomId, LocalUserId, ChatMessageId};

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = chat_message))]
#[cfg_attr(feature = "full", diesel(primary_key(id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct ChatMessage {
    pub id: ChatMessageId,
    pub room_id: ChatRoomId,
    pub sender_id: LocalUserId,
    pub content: String,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(
    feature = "full",
    derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = chat_message))]
pub struct ChatMessageInsertForm {
    pub room_id: ChatRoomId,
    pub sender_id: LocalUserId,
    pub content: String,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
#[derive(Debug, Clone, Serialize)]
pub enum ChatMessageContent {
    Text { content: String },
    File { name: String, file_type: String },
}

impl From<String> for ChatMessageContent {
    fn from(content: String) -> Self {
        if content.starts_with("file:") {
            ChatMessageContent::File {
                name: content,
                file_type: "".to_string(),
            }
        } else {
            ChatMessageContent::Text { content }
        }
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(Serialize, Deserialize, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = chat_message))]
pub struct ChatMessageUpdateForm {
    pub content: String,
    pub status: Option<i16>,
    pub updated_at: Option<DateTime<Utc>>,
}