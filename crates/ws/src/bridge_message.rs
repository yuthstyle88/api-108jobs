use actix::Message;
use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct BridgeMessage {
    pub channel: ChatRoomId,
    pub event: String,
    pub messages: Option<String>,
    pub security_config: bool,
}
#[skip_serializing_none]
#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct OutboundMessage {
    pub channel: ChatRoomId,
    pub event: String,
    pub messages: Option<String>,
    pub security_config: bool,
}