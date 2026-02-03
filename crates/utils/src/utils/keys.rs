/// Redis & pubsub keys for presence, contacts, rooms, and pubsub topics.

#[inline]
pub fn contacts_key(user_id: i32) -> String {
  format!("contacts:user:{}", user_id)
}

#[inline]
pub fn rooms_key(user_id: i32) -> String {
  format!("rooms:user:{}", user_id)
}

#[inline]
pub fn presence_conn_key(user_id: i32, conn_id: &str) -> String {
  format!("presence:user:{}:conn:{}", user_id, conn_id)
}

#[inline]
pub fn presence_conn_count_key(user_id: i32) -> String {
  format!("presence:user:{}:conn_count", user_id)
}

#[inline]
pub fn presence_conn_pattern(user_id: i32) -> String {
  format!("presence:user:{}:conn:*", user_id)
}

#[inline]
pub fn user_events_topic(user_id: &str) -> String {
  format!("user:{}:events", user_id)
}

#[inline]
pub fn room_topic(room_id: &str) -> String {
  format!("room:{}", room_id)
}
