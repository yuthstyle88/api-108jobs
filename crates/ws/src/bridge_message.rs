use actix::Message;
use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId};
use serde::{Deserialize, Serialize};

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