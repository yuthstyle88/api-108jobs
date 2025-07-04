use actix::prelude::*;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ChatMessage(pub String);

#[derive(Message)]
#[rtype(result = "u64")]
pub struct JoinRoom(pub String, pub Option<String>, pub Recipient<ChatMessage>);

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct LeaveRoom(pub String, pub u64);

#[derive(Message, Clone)]
#[rtype(result = "Vec<String>")]
pub struct ListRooms;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct SendMessage(pub String, pub u64, pub String);