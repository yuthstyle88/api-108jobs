use actix::Message;
use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId};
use serde::{Deserialize, Serialize};

#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct BridgeMessage {
    pub channel: ChatRoomId,
    pub local_user_id: LocalUserId,
    pub event: String,
    pub messages: String,
    pub security_config: bool,
}

#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct OnlineJoin {
    pub channel: ChatRoomId,
    pub local_user_id: LocalUserId,
}

#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct OnlineLeave {
    pub channel: ChatRoomId,
    pub local_user_id: LocalUserId,
}

#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct OnlinePing {
    pub channel: ChatRoomId,
    pub local_user_id: LocalUserId,
}