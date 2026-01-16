use crate::bridge_message::{GlobalOffline, GlobalOnline};
use crate::broker::bridge_message::EmitTopics;
use crate::broker::manager::{GetPresenceSnapshot, PhoenixManager};
use crate::protocol::api::{ChatEvent, ChatsSignalPayload, PresenceSnapshotItem, PresenceStatus};
use actix::{Actor, Context, Handler, Message};
use actix::{Addr, ResponseFuture};
use actix_broker::{BrokerIssue, SystemBroker};
use app_108jobs_db_schema::newtypes::{ChatRoomId, LocalUserId};
use app_108jobs_db_schema::source::chat_participant::ChatParticipant;
use app_108jobs_db_schema::utils::{ActualDbPool, DbPool};
use app_108jobs_utils::error::FastJobResult;
use app_108jobs_utils::redis::{AsyncCommands, RedisClient};
use app_108jobs_utils::utils::helper::{
  contacts_key, presence_conn_count_key, presence_conn_key, rooms_key,
  user_events_topic,
};
use chrono::{DateTime, Utc};
use serde_json::json;
use std::collections::HashSet;
use std::time::Duration;
use tracing;

/// ===== PresenceManager Actor =====

/// Tracks online presence using heartbeats and explicit joins/leaves.
/// Emits OnlineStopped when a user misses heartbeats beyond `heartbeat_ttl`.
pub struct PresenceManager {
  /// How long we wait before declaring a user “stopped” (timeout).
  heartbeat_ttl: Duration,
  redis: Option<RedisClient>,
  pool: ActualDbPool,
  /// Track which rooms each user is active in (for broadcasting presence to partners)
  rooms_by_user: std::collections::HashMap<i32, HashSet<ChatRoomId>>,
  phoenix_addr: Option<Addr<PhoenixManager>>,
}

/// ===== Presence messages =====
#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct OnlineJoin {
  pub room_id: ChatRoomId,
  pub local_user_id: LocalUserId,
  pub started_at: DateTime<Utc>,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct OnlineLeave {
  pub room_id: ChatRoomId,
  pub local_user_id: LocalUserId,
  pub left_at: DateTime<Utc>,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct OnlineStopped {
  pub local_user_id: LocalUserId,
  pub stopped_at: DateTime<Utc>,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct Heartbeat {
  pub local_user_id: LocalUserId,
  pub connection_id: String,
  pub client_time: Option<DateTime<Utc>>,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "bool")]
pub struct IsUserOnline {
  pub local_user_id: LocalUserId,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "usize")]
pub struct OnlineCount;

#[derive(Message)]
#[rtype(result = "()")]
pub struct AttachPhoenix {
  pub addr: Addr<PhoenixManager>,
}

impl Handler<AttachPhoenix> for PresenceManager {
  type Result = ();

  fn handle(&mut self, msg: AttachPhoenix, _: &mut Context<Self>) {
    self.phoenix_addr = Some(msg.addr);
  }
}

async fn ensure_contacts_loaded(
  redis: &mut RedisClient,
  db_pool: ActualDbPool,
  user_id: LocalUserId,
) {
  let key = format!("contacts:user:{}", user_id.0);

  // Already cached → do nothing
  if redis.exists(&key).await.unwrap_or(false) {
    return;
  }

  // CLONE redis for async task
  let mut redis = redis.clone();

  actix::spawn(async move {
    let mut db = DbPool::Pool(&db_pool);

    let Ok((rooms, members_by_room)) =
      ChatParticipant::load_user_rooms_and_members(&mut db, user_id).await
    else {
      return;
    };

    let _ = sync_user_contacts_from_db(&mut redis, user_id, rooms, members_by_room).await;
  });
}

/// One-time sync of user rooms + contacts from DB into Redis.
/// This should be called ONLY when Redis cache is missing (cold start).
///
/// Redis keys:
/// - rooms:user:{user_id}      -> SET(room_id)
/// - contacts:user:{user_id}   -> SET(member_id)
pub async fn sync_user_contacts_from_db(
  redis: &mut RedisClient,
  user_id: LocalUserId,
  rooms: Vec<ChatRoomId>,
  members_by_room: Vec<(ChatRoomId, Vec<LocalUserId>)>,
) -> FastJobResult<()> {
  let user_rooms_key = rooms_key(user_id.0);
  let user_contacts_key = contacts_key(user_id.0);

  // Optional: clear stale keys before rebuilding
  // (safe because this is cold-sync only)
  let _ = redis.delete_key(&user_rooms_key).await;
  let _ = redis.delete_key(&user_contacts_key).await;

  // Pipeline for best performance
  let mut pipe = redis.pipeline();

  // Save rooms
  for room in &rooms {
    pipe.sadd(&user_rooms_key, room.0.as_str());
  }

  // Save contacts (all other members from those rooms)
  for (_room_id, members) in members_by_room {
    for member_id in members {
      if member_id != user_id {
        pipe.sadd(&user_contacts_key, member_id.0);
      }
    }
  }

  // Execute pipeline
  redis.exec_pipeline(&mut pipe).await?;

  Ok(())
}

impl PresenceManager {
  pub fn new(heartbeat_ttl: Duration, redis: Option<RedisClient>, pool: ActualDbPool) -> Self {
    Self {
      heartbeat_ttl,
      redis,
      pool,
      rooms_by_user: Default::default(),
      phoenix_addr: None,
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

fn presence_diff_join(room_id: ChatRoomId, user_id: i32, at: DateTime<Utc>) -> serde_json::Value {
  json!({
    "type": "presence:diff",
    "room_id": room_id,
    "joins": [{ "user_id": user_id, "at": at }],
    "leaves": []
  })
}

fn presence_diff_leave(
  room_id: ChatRoomId,
  user_id: i32,
  last_seen: DateTime<Utc>,
) -> serde_json::Value {
  json!({
    "type": "presence:diff",
    "room_id": room_id,
    "joins": [],
    "leaves": [{ "user_id": user_id, "last_seen": last_seen }]
  })
}

impl Handler<OnlineJoin> for PresenceManager {
  type Result = ();
  fn handle(&mut self, msg: OnlineJoin, _ctx: &mut Context<Self>) -> Self::Result {
    // Track room membership and broadcast join
    let rooms = self.rooms_by_user.entry(msg.local_user_id.0).or_default();

    if rooms.insert(msg.room_id.clone()) {
      let topic = format!("room:{}", msg.room_id);
      let payload = presence_diff_join(msg.room_id, msg.local_user_id.0, msg.started_at);
      self.issue_async::<SystemBroker, _>(EmitTopics {
        items: vec![(topic, ChatEvent::ChatsSignal, payload)],
      });
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
      let payload = presence_diff_leave(msg.room_id, msg.local_user_id.0, msg.left_at);
      self.issue_async::<SystemBroker, _>(EmitTopics {
        items: vec![(topic, ChatEvent::ChatsSignal, payload)],
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
          let payload = presence_diff_leave(room_id, msg.local_user_id.0, stopped_at);

          (topic, ChatEvent::ChatsSignal, payload)
        })
        .collect();
      if !items.is_empty() {
        self.issue_async::<SystemBroker, _>(EmitTopics { items });
      }
    }
  }
}

async fn emit_presence_to_contacts(
  phoenix: &Addr<PhoenixManager>,
  redis: &mut RedisClient,
  user_id: i32,
  payload: serde_json::Value,
) -> FastJobResult<()> {
  let contact_ids = redis.smembers(&contacts_key(user_id)).await?;

  if contact_ids.is_empty() {
    return Ok(());
  }

  let items = contact_ids
    .into_iter()
    .map(|cid| {
      (
        user_events_topic(&cid),
        ChatEvent::ChatsSignal,
        payload.clone(),
      )
    })
    .collect();

  phoenix.do_send(EmitTopics { items });
  Ok(())
}

impl Handler<GlobalOnline> for PresenceManager {
  type Result = ResponseFuture<()>;

  fn handle(&mut self, msg: GlobalOnline, _ctx: &mut Context<Self>) -> Self::Result {
    let redis = self.redis.clone();
    let db_pool = self.pool.clone();
    let ttl = self.heartbeat_ttl.as_secs() as usize;
    let user_id = msg.local_user_id.0;
    let conn_id = msg.connection_id.clone();
    let started_at = msg.at;
    let phoenix = self.phoenix_addr.clone();

    Box::pin(async move {
      let (Some(mut client), Some(phoenix)) = (redis, phoenix) else {
        return;
      };

      ensure_contacts_loaded(&mut client, db_pool, msg.local_user_id).await;

      let conn_key = presence_conn_key(user_id, &conn_id);

      // set connection with TTL
      let _ = client.set_value_with_expiry(&conn_key, 1, ttl).await;

      let count: i64 = client
        .connection
        .incr(presence_conn_count_key(user_id), 1)
        .await
        .unwrap_or(0);

      // FIRST connection only
      if count == 1 {
        let payload = serde_json::to_value(ChatsSignalPayload::GlobalPresence {
          version: 1,
          user_id: msg.local_user_id,
          status: PresenceStatus::Online,
          at: started_at,
          last_seen: None,
        })
        .unwrap();

        let _ = emit_presence_to_contacts(&phoenix, &mut client, user_id, payload).await;
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
    let phoenix = self.phoenix_addr.clone();

    Box::pin(async move {
      let (Some(mut client), Some(phoenix)) = (redis, phoenix) else {
        return;
      };

      let conn_key = presence_conn_key(user_id, &conn_id);
      let _ = client.delete_key(&conn_key).await;

      let count: i64 = client
        .connection
        .decr(presence_conn_count_key(user_id), 1)
        .await
        .unwrap_or(0);

      // LAST connection only
      if count <= 0 {
        let payload = serde_json::to_value(ChatsSignalPayload::GlobalPresence {
          version: 1,
          user_id: msg.local_user_id,
          status: PresenceStatus::Offline,
          at: Utc::now(),
          last_seen: Some(Utc::now()),
        })
        .unwrap();

        let _ = emit_presence_to_contacts(&phoenix, &mut client, user_id, payload).await;
      }
    })
  }
}

impl Handler<Heartbeat> for PresenceManager {
  type Result = ();
  fn handle(&mut self, msg: Heartbeat, _ctx: &mut Context<Self>) -> Self::Result {
    if let Some(mut client) = self.redis.clone() {
      let ttl = self.heartbeat_ttl.as_secs() as usize;
      let conn_key = presence_conn_key(msg.local_user_id.0, &msg.connection_id);
      actix::spawn(async move {
        let _ = client.expire(&conn_key, ttl).await;
      });
    }
  }
}

impl Handler<IsUserOnline> for PresenceManager {
  type Result = ResponseFuture<bool>;

  fn handle(&mut self, msg: IsUserOnline, _ctx: &mut Context<Self>) -> Self::Result {
    let redis = self.redis.clone();

    Box::pin(async move {
      if let Some(mut client) = redis {
        client
          .get_value::<i64>(&presence_conn_count_key(msg.local_user_id.0))
          .await
          .unwrap_or(Some(0))
          > Some(0)
      } else {
        false
      }
    })
  }
}

impl Handler<GetPresenceSnapshot> for PresenceManager {
  type Result = ResponseFuture<FastJobResult<Vec<PresenceSnapshotItem>>>;

  fn handle(&mut self, msg: GetPresenceSnapshot, _ctx: &mut Context<Self>) -> Self::Result {
    let redis = self.redis.clone();
    let requester_id = msg.local_user_id.0;

    Box::pin(async move {
      let Some(mut redis) = redis else {
        return Ok(Vec::new());
      };

      // get contacts
      let contacts_key = contacts_key(requester_id);
      let contacts: Vec<String> = redis.smembers(&contacts_key).await?;

      let mut items = Vec::with_capacity(contacts.len());
      let now = Utc::now();

      // presence check (same logic as IsUserOnline)
      for cid in contacts {
        let Ok(cid) = cid.parse::<i32>() else {
          continue;
        };

        // Use the same approach as IsUserOnline: check the aggregated connection count key
        let online = redis
          .get_value::<i64>(&presence_conn_count_key(cid))
          .await
          .unwrap_or(Some(0))
          > Some(0);

        if online {
          items.push(PresenceSnapshotItem {
            user_id: LocalUserId(cid),
            status: PresenceStatus::Online,
            at: now,
            last_seen: None,
          });
        }
      }

      Ok(items)
    })
  }
}
