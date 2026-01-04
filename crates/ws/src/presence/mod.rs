use actix::{Actor, Context, Handler, Message};
use actix::ResponseFuture;
use chrono::{DateTime, Utc};
use std::time::Duration;
use std::collections::HashSet;
use tracing;
use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId};
use lemmy_utils::redis::RedisClient;
use actix_broker::{Broker, BrokerIssue, SystemBroker};
use serde_json::json;

use crate::bridge_message::{GlobalOffline, GlobalOnline};
use crate::protocol::api::ChatEvent;
use crate::broker::bridge_message::EmitTopics;

// Note: we use actix_broker::SystemBroker; do not redefine here.

/// ===== PresenceManager Actor =====

/// Tracks online presence using heartbeats and explicit joins/leaves.
/// Emits OnlineStopped when a user misses heartbeats beyond `heartbeat_ttl`.
pub struct PresenceManager {
    /// How long we wait before declaring a user “stopped” (timeout).
    heartbeat_ttl: Duration,
    redis: Option<RedisClient>,
    /// Track which rooms each user is active in (for broadcasting presence to partners)
    rooms_by_user: std::collections::HashMap<i32, std::collections::HashSet<ChatRoomId>>,    
}

/// ===== Presence messages =====
#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct OnlineJoin { pub room_id: ChatRoomId, pub local_user_id: LocalUserId, pub started_at: DateTime<Utc> }

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct OnlineLeave {pub room_id: ChatRoomId, pub local_user_id: LocalUserId, pub left_at: DateTime<Utc> }

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct OnlineStopped {pub local_user_id: LocalUserId, pub stopped_at: DateTime<Utc> }

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct Heartbeat {
    pub local_user_id: LocalUserId,
    pub connection_id: String,
    pub client_time: Option<DateTime<Utc>>,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "bool")]
pub struct IsUserOnline {pub local_user_id: LocalUserId }

#[derive(Message, Clone, Debug)]
#[rtype(result = "usize")]
pub struct OnlineCount;

impl PresenceManager {
    pub fn new(
        heartbeat_ttl: Duration,
        redis: Option<RedisClient>,
    ) -> Self {
        Self {
            heartbeat_ttl,
            redis,
            rooms_by_user: Default::default(),
        }
    }
}

impl Actor for PresenceManager {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::info!(
            ttl = self.heartbeat_ttl.as_secs(),
            has_redis = self.redis.is_some(),
            "PresenceManager started"
        );
    }
}

impl Handler<OnlineJoin> for PresenceManager {
    type Result = ();
    fn handle(&mut self, msg: OnlineJoin, _ctx: &mut Context<Self>) -> Self::Result {
        // Track room membership and broadcast join
        let rooms = self
            .rooms_by_user
            .entry(msg.local_user_id.0)
            .or_default();

        if rooms.insert(msg.room_id.clone()) {
            let topic = format!("room:{}", msg.room_id);
            let payload = json!({
                "type": "presence:diff",
                "room_id": msg.room_id,
                "joins": [{"user_id": msg.local_user_id.0, "at": msg.started_at} ],
                "leaves": []
            });
            self.issue_async::<SystemBroker, _>(EmitTopics { items: vec![(topic, ChatEvent::Update, payload)]});
        }
    }
}

impl Handler<OnlineLeave> for PresenceManager {
    type Result = ();
    fn handle(&mut self, msg: OnlineLeave, _ctx: &mut Context<Self>) -> Self::Result {
        let should_broadcast = if let Some(rooms) = self.rooms_by_user.get_mut(&msg.local_user_id.0) {
            rooms.remove(&msg.room_id)
        } else {
            false
        };

        if should_broadcast {
            let topic = format!("room:{}", msg.room_id);
            let payload = json!({
                "type": "presence:diff",
                "room_id": msg.room_id,
                "joins": [],
                "leaves": [{
                    "user_id": msg.local_user_id.0,
                    "last_seen": msg.left_at
                }]
            });
            self.issue_async::<SystemBroker, _>(EmitTopics {
                items: vec![(topic, ChatEvent::Update, payload)]
            });
        }

        // Clean up empty user entry
        if let Some(rooms) = self.rooms_by_user.get(&msg.local_user_id.0) {
            if rooms.is_empty() {
                self.rooms_by_user.remove(&msg.local_user_id.0);
            }
        }
    }
}

impl Handler<OnlineStopped> for PresenceManager {
    type Result = ();
    fn handle(&mut self, msg: OnlineStopped, _ctx: &mut Context<Self>) -> Self::Result {
        // Broadcast leaves to all rooms this user was active in
        if let Some(rooms) = self.rooms_by_user.remove(&msg.local_user_id.0) {
            let stopped_at = msg.stopped_at;
            let items: Vec<(String, ChatEvent, serde_json::Value)> = rooms
                .into_iter()
                .map(|room_id| {
                    let topic = format!("room:{}", room_id);
                    let payload = json!({
                        "type": "presence:diff",
                        "room_id": room_id,
                        "joins": [],
                        "leaves": [{"user_id": msg.local_user_id.0, "last_seen": stopped_at} ]
                    });
                    (topic, ChatEvent::Update, payload)
                })
                .collect();
            if !items.is_empty() {
                self.issue_async::<SystemBroker, _>(EmitTopics { items });
            }
        }
        // tracing::info!(local_user_id = msg.local_user_id, ts = %msg.stopped_at, "presence: online_stopped(event)");
    }
}

impl Handler<GlobalOnline> for PresenceManager {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: GlobalOnline, _ctx: &mut Context<Self>) -> Self::Result {
        let redis = self.redis.clone();
        let ttl = self.heartbeat_ttl.as_secs() as usize;
        let user_id = msg.local_user_id.0;
        let conn_id = msg.connection_id.clone();
        let started_at = msg.at;

        Box::pin(async move {
            if let Some(mut client) = redis {
                let conn_key = format!("presence:user:{}:conn:{}", user_id, conn_id);
                // Set connection key with TTL
                let _ = client.set_value_with_expiry(&conn_key, 1, ttl).await;

                // Check if this is the first connection
                let pattern = format!("presence:user:{}:conn:*", user_id);
                if let Ok(keys) = client.keys(&pattern).await {
                    if keys.len() == 1 {
                        // First connection -> Broadcast online
                        Broker::<SystemBroker>::issue_async(EmitTopics {
                            items: vec![(
                                format!("presence:user:{}", user_id),
                                ChatEvent::Update,
                                json!({
                                    "type": "presence",
                                    "user_id": user_id,
                                    "status": "online",
                                    "at": started_at
                                })
                            )]
                        });
                    }
                }
            }
        })
    }
}

impl Handler<GlobalOffline> for PresenceManager {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: GlobalOffline, _ctx: &mut Context<Self>) -> Self::Result {
        let redis = self.redis.clone();
        let user_id = msg.local_user_id.0;
        let conn_id = msg.connection_id.clone();

        Box::pin(async move {
            if let Some(mut client) = redis {
                let conn_key = format!("presence:user:{}:conn:{}", user_id, conn_id);
                let _ = client.delete_key(&conn_key).await;

                // Check if this was the last connection
                let pattern = format!("presence:user:{}:conn:*", user_id);
                if let Ok(keys) = client.keys(&pattern).await {
                    if keys.is_empty() {
                        // Last connection -> Broadcast offline
                        Broker::<SystemBroker>::issue_async(EmitTopics {
                            items: vec![(
                                format!("presence:user:{}", user_id),
                                ChatEvent::Update,
                                json!({
                                    "type": "presence",
                                    "user_id": user_id,
                                    "status": "offline",
                                    "last_seen": Utc::now()
                                })
                            )]
                        });
                    }
                }
            }
        })
    }
}

impl Handler<Heartbeat> for PresenceManager {
    type Result = ();
    fn handle(&mut self, msg: Heartbeat, _ctx: &mut Context<Self>) -> Self::Result {
        if let Some(mut client) = self.redis.clone() {
            let ttl = self.heartbeat_ttl.as_secs() as usize;
            let conn_key = format!("presence:user:{}:conn:{}", msg.local_user_id.0, msg.connection_id);
            actix::spawn(async move {
                let _ = client.expire(&conn_key, ttl).await;
            });
        }
    }
}

impl Handler<IsUserOnline> for PresenceManager {
    type Result = ResponseFuture<bool>;
    fn handle(&mut self, msg: IsUserOnline, _ctx: &mut Context<Self>) -> Self::Result {
        let client = self.redis.clone();
        Box::pin(async move {
            if let Some(mut client) = client {
                let pattern = format!("presence:user:{}:conn:*", msg.local_user_id.0);
                if let Ok(keys) = client.keys(&pattern).await {
                    return !keys.is_empty();
                }
            }
            false
        })
    }
}

impl Handler<OnlineCount> for PresenceManager {
    type Result = ResponseFuture<usize>;
    fn handle(&mut self, _msg: OnlineCount, _ctx: &mut Context<Self>) -> Self::Result {
        let client = self.redis.clone();
        Box::pin(async move {
            if let Some(mut client) = client {
                // This is a bit expensive but accurate for global online count across all devices
                // We want count of unique users who have at least one connection key
                let pattern = "presence:user:*:conn:*".to_string();
                if let Ok(keys) = client.keys(&pattern).await {
                    let unique_users: HashSet<String> = keys
                        .into_iter()
                        .filter_map(|k| k.split(':').nth(2).map(|s| s.to_string()))
                        .collect();
                    return unique_users.len();
                }
            }
            0
        })
    }
}