use crate::session::Session;
use log::{error, info};
use std::{
  collections::{HashMap, HashSet},
  sync::Arc,
};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Main proxy manager for Phoenix integration
#[derive(Clone)]
pub struct PhoenixProxy {
  clients: Arc<RwLock<HashMap<String, Vec<Session>>>>,
  user_sessions: Arc<RwLock<HashMap<Uuid, Vec<Session>>>>,
  user_rooms: Arc<RwLock<HashMap<Uuid, HashSet<String>>>>,
  sender: mpsc::Sender<(String, String)>,
}

impl PhoenixProxy {
  pub fn new(sender: mpsc::Sender<(String, String)>) -> Self {
    Self {
      clients: Arc::new(RwLock::new(HashMap::new())),
      user_sessions: Arc::new(RwLock::new(HashMap::new())),
      user_rooms: Arc::new(RwLock::new(HashMap::new())),
      sender,
    }
  }

  pub async fn add_client(&self, room_id: &str, user_id: Uuid, session: Session) {
    {
      let mut user_sessions = self.user_sessions.write().await;
      user_sessions
        .entry(user_id)
        .or_default()
        .push(session.clone());
    }

    {
      let mut clients = self.clients.write().await;
      clients
        .entry(room_id.to_string())
        .or_default()
        .push(session.clone());
    }

    {
      let mut user_rooms = self.user_rooms.write().await;
      user_rooms
        .entry(user_id)
        .or_default()
        .insert(room_id.to_string());
    }

    info!("Client added to room {} for user {}", room_id, user_id);
  }

  pub async fn send_message_to_phoenix(&self, room_id: String, message: String) {
    let _ = self.sender.send((room_id, message)).await;
  }

  pub async fn broadcast_to_room(&self, room_id: &str, message: &str) {
    let clients = self.clients.read().await;
    if let Some(sessions) = clients.get(room_id) {
      for session in sessions {
        if let Err(e) = session.text(message.to_string()).await {
          error!("Failed to send to session: {:?}", e);
        }
      }
    }
  }

  pub async fn notify_user(&self, user_id: Uuid, notification: &str) {
    let user_sessions = self.user_sessions.read().await;
    if let Some(sessions) = user_sessions.get(&user_id) {
      for session in sessions {
        if let Err(e) = session.text(notification.to_string()).await {
          error!("Failed to notify user {}: {:?}", user_id, e);
        }
      }
    }
  }

  pub async fn add_user_rooms_client(&self, user_id: Uuid, rooms: Vec<Uuid>, session: Session) {
    {
      let mut user_sessions = self.user_sessions.write().await;
      user_sessions
        .entry(user_id)
        .or_default()
        .push(session.clone());
    }

    let mut room_ids: Vec<String> = rooms.iter().map(|id| id.to_string()).collect();
    if rooms.iter().any(|id| id == &Uuid::nil()) {
      room_ids.push("all_rooms".into());
    }

    {
      let mut user_rooms = self.user_rooms.write().await;
      let room_set = user_rooms.entry(user_id).or_default();
      for room_id in &room_ids {
        room_set.insert(room_id.clone());

        let mut clients = self.clients.write().await;
        clients
          .entry(room_id.clone())
          .or_default()
          .push(session.clone());
      }
    }

    // Send initial room list
    let room_list = serde_json::json!({
        "type": "room_list",
        "rooms": room_ids,
    });

    if let Ok(msg) = serde_json::to_string(&room_list) {
      let _ = session.text(msg).await;
    }
  }
}
