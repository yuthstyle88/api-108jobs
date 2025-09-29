// presence.rs
use actix::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};
use lemmy_db_schema::newtypes::LocalUserId;

/// ===== Messages / Events =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineJoin {
  /// Unique user identifier.
  pub local_user_id: Option<LocalUserId>,
  /// When we detected they came online (server time).
  pub started_at: DateTime<Utc>,
}

impl Message for OnlineJoin {
  type Result = ();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineLeave {
  /// Explicit/normal leave (user closed session, logged out, etc.)
  pub local_user_id: Option<LocalUserId>,
  /// When we detected they left (server time).
  pub left_at: DateTime<Utc>,
}

impl Message for OnlineLeave {
  type Result = ();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineStopped {
  /// Abnormal stop detected by heartbeat timeout (no signal).
  pub local_user_id: Option<LocalUserId>,
  /// When we detected the stop (server time).
  pub stopped_at: DateTime<Utc>,
}

impl Message for OnlineStopped {
  type Result = ();
}

/// Heartbeat ping from client (or edge gateway).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
  pub local_user_id: Option<LocalUserId>,
  /// Optional client-reported time; server will still record server time.
  pub client_time: Option<DateTime<Utc>>,
}

impl Message for Heartbeat {
  type Result = ();
}

/// Optional: query who is online (useful for API handlers).
#[derive(Message)]
#[rtype(result = "HashSet<i32>")]
pub struct GetOnlineUsers;

/// ===== PresenceManager Actor =====

/// Tracks online presence using heartbeats and explicit joins/leaves.
/// Emits OnlineStopped when a user misses heartbeats beyond `heartbeat_ttl`.
pub struct PresenceManager {
  /// Last-seen (server time) per user.
  last_seen: HashMap<i32, DateTime<Utc>>,
  /// Known-online users (authoritative “online” set).
  online_users: HashSet<i32>,
  /// How long we wait before declaring a user “stopped” (timeout).
  heartbeat_ttl: Duration,
  /// How often the sweeper runs to check timeouts.
  sweep_interval: Duration,
  /// Optional callback (e.g., to Redis/Phoenix broker). Keep it simple: an Addr to another actor.
  broker: Option<Addr<SystemBroker>>,
}

/// Minimal broker example. Replace with your real broker / Phoenix bridge.
pub struct SystemBroker;

impl Actor for SystemBroker {
  type Context = Context<Self>;
}

/// Light-weight publish API. Replace with your real event schema/bridge.
#[derive(Message)]
#[rtype(result = "()")]
pub struct PublishPresenceEvent {
  pub topic: String,
  pub payload: serde_json::Value,
}

impl Handler<PublishPresenceEvent> for SystemBroker {
  type Result = ();

  fn handle(&mut self, msg: PublishPresenceEvent, _ctx: &mut Self::Context) -> Self::Result {
    // TODO: bridge to Phoenix channel or WebSocket fanout.
    // e.g., self.phoenix.push(&msg.topic, "presence_event", msg.payload)
    // Avoid printing logs here per your preference; keep structured logs only if you want.
    let _ = (msg.topic, msg.payload);
  }
}

impl PresenceManager {
  pub fn new(
    heartbeat_ttl: Duration,
    sweep_interval: Duration,
    broker: Option<Addr<SystemBroker>>,
  ) -> Self {
    Self {
      last_seen: HashMap::new(),
      online_users: HashSet::new(),
      heartbeat_ttl,
      sweep_interval,
      broker,
    }
  }

  /// Helper: record last seen “now”.
  fn touch(&mut self, user_id: i32) -> DateTime<Utc> {
    let now = DateTime::<Utc>::from(SystemTime::now());
    self.last_seen.insert(user_id, now);
    now
  }

  /// Helper: publish to broker if configured.
  fn publish<T: Serialize>(&self, topic: &str, event: &str, payload: &T) {
    if let Some(broker) = &self.broker {
      if let Ok(value) = serde_json::to_value(payload) {
        let wrapped = serde_json::json!({
            "type": event,
            "data": value
        });
        broker.do_send(PublishPresenceEvent {
          topic: topic.to_string(),
          payload: wrapped,
        });
      }
    }
  }

  /// Helper: mark user offline internally and remove last_seen.
  fn mark_offline(&mut self, local_user_id: LocalUserId) {
    self.online_users.remove(&local_user_id.0);
    self.last_seen.remove(&local_user_id.0);
  }

  /// Helper: mark user online internally.
  fn mark_online(&mut self, local_user_id: LocalUserId) {
    self.online_users.insert(local_user_id.0);
  }

  /// Sweeper that detects timeouts and emits OnlineStopped.
  fn sweep_timeouts(&mut self, ctx: &mut <Self as Actor>::Context) {
    let now = DateTime::<Utc>::from(SystemTime::now());
    let ttl = chrono::Duration::from_std(self.heartbeat_ttl)
      .unwrap_or_else(|_| chrono::Duration::seconds(30));

    // Collect offenders first to avoid double-borrow of self
    let offenders: Vec<i32> = self
      .online_users
      .iter()
      .copied()
      .filter(|uid| {
        if let Some(seen) = self.last_seen.get(uid) {
          now.signed_duration_since(*seen) > ttl
        } else {
          true
        }
      })
      .collect();

    for uid in offenders {
      let stopped = OnlineStopped {
        local_user_id: Some(LocalUserId(uid)),
        stopped_at: now,
      };

      // Update state
      self.mark_offline(LocalUserId(uid));

      // Publish/broadcast
      self.publish("presence", "online_stopped", &stopped);

      // Also notify internally if other actors care
      ctx.notify(stopped);
    }
  }
}

impl Actor for PresenceManager {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    // Periodic timeout sweeper
    let every = self.sweep_interval;
    ctx.run_interval(every, |actor, ctx| {
      actor.sweep_timeouts(ctx);
    });
  }
}

/// === Handlers ===

impl Handler<OnlineJoin> for PresenceManager {
  type Result = ();

  fn handle(&mut self, msg: OnlineJoin, _ctx: &mut Self::Context) -> Self::Result {
    let local_user_id = msg.local_user_id;
    if let Some(uid) = local_user_id{
      self.mark_online(uid);
      let uid = uid.0;
      self.last_seen.insert(uid, msg.started_at);
    }
    self.publish("presence", "online_join", &msg);
  }
}

impl Handler<OnlineLeave> for PresenceManager {
  type Result = ();

  fn handle(&mut self, msg: OnlineLeave, _ctx: &mut Self::Context) -> Self::Result {
    if let Some(uid) = msg.local_user_id {
      // Authoritatively mark the user offline in Presence only
      self.mark_offline(uid);
    }
    // Publish presence event (idempotent)
    self.publish("presence", "online_leave", &msg);
  }
}

impl Handler<OnlineStopped> for PresenceManager {
  type Result = ();

  fn handle(&mut self, msg: OnlineStopped, _ctx: &mut Self::Context) -> Self::Result {
    // Idempotent: ensure offline
    let local_user_id = msg.local_user_id;
    if let Some(uid) = local_user_id {
      self.mark_offline(uid);
    }
    self.publish("presence", "online_stopped", &msg);
  }
}

impl Handler<Heartbeat> for PresenceManager {
  type Result = ();

  fn handle(&mut self, msg: Heartbeat, _ctx: &mut Self::Context) -> Self::Result {
    if let Some(uid) = msg.local_user_id {
      // Record server-side last_seen regardless of client_time
      let now = self.touch(uid.0);

      // If user not already in the authoritative online set, promote on first heartbeat
      if !self.online_users.contains(&uid.0) {
        self.mark_online(uid);
        let join = OnlineJoin {
          local_user_id: Some(uid),
          started_at: now,
        };
        self.publish("presence", "online_join", &join);
      }
    }
  }
}

impl Handler<GetOnlineUsers> for PresenceManager {
  type Result = MessageResult<GetOnlineUsers>;

  fn handle(&mut self, _msg: GetOnlineUsers, _ctx: &mut Self::Context) -> Self::Result {
    MessageResult(self.online_users.clone())
  }
}
