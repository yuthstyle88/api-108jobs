use actix::Message;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use crate::api::{IncomingEvent};

#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct BridgeMessage {
    pub incoming_event: IncomingEvent, 
}
#[skip_serializing_none]
#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct OutboundMessage {
    pub out_event: IncomingEvent,
}