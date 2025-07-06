use actix::prelude::*;
use lemmy_db_schema::newtypes::{ClientKey, LocalUserId};
use lemmy_db_schema::source::chat_message::ChatMessageInsertForm;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct LeaveRoom(pub String, pub u64);

#[derive(Message, Clone)]
#[rtype(result = "Vec<String>")]
pub struct ListRooms;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct SendMessage(pub String, pub u64, pub String);

#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct StoreChatMessage {
    pub message: ChatMessageInsertForm,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct RegisterClientKeyMsg {
    pub user_id: Option<LocalUserId>,
    pub client_key: Option<String>
}