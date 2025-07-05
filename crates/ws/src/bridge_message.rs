use actix::Message;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize)]
pub enum MessageSource {
    WebSocket,
    Phoenix,
}

#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct BridgeMessage {
    pub op: String,
    pub source: MessageSource,
    pub channel: String,
    pub event: String,
    pub messages: String,
}