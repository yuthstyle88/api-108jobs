use crate::api::IncomingEvent;
use actix::Message;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use crate::impls::AnyIncomingEvent;

#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct BridgeMessage {
    pub any_event: AnyIncomingEvent,
    pub incoming_event: IncomingEvent,
}
#[skip_serializing_none]
#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct OutboundMessage {
    pub out_event: IncomingEvent,
}