use actix::Message;
use serde::{Deserialize, Serialize};
use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId};

#[derive(Clone, Serialize, Deserialize)]
pub enum MessageSource {
    WebSocket,
    Phoenix,
}

#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct BridgeMessage {
    pub source: MessageSource,
    pub channel: ChatRoomId,
    pub user_id: LocalUserId,
    pub event: String,
    pub messages: String,
    pub security_config: bool,
}