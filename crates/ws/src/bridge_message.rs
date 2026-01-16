use crate::protocol::api::IncomingEvent;
use actix::Message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use app_108jobs_db_schema::newtypes::LocalUserId;
use crate::protocol::impls::AnyIncomingEvent;

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

#[derive(Message, Clone, Debug, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct GlobalOnline {
    pub local_user_id: LocalUserId,
    pub connection_id: String,
    pub at: DateTime<Utc>,
}

#[derive(Message, Clone, Debug, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct GlobalOffline {
    pub local_user_id: LocalUserId,
    pub connection_id: String,
}
