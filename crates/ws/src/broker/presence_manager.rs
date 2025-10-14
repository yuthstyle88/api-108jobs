use actix::{Actor, AsyncContext, Context, Handler, Message};
use actix::ResponseFuture;
use chrono::{DateTime, Utc};
use std::time::{Duration as StdDuration, Duration};
use std::collections::HashSet;
use tracing;
use lemmy_db_schema::newtypes::{ChatRoomId, LocalUserId};
use lemmy_utils::redis::RedisClient;

pub struct SystemBroker;

impl Actor for SystemBroker {
    type Context = Context<Self>;
}

/// ===== PresenceManager Actor =====

/// Tracks online presence using heartbeats and explicit joins/leaves.
/// Emits OnlineStopped when a user misses heartbeats beyond `heartbeat_ttl`.
pub struct PresenceManager {
    /// How long we wait before declaring a user “stopped” (timeout).
    heartbeat_ttl: Duration,
    redis: Option<RedisClient>,
    local_online: HashSet<i32>,
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
pub struct OnlineStopped {pub room_id: ChatRoomId, pub local_user_id: LocalUserId, pub stopped_at: DateTime<Utc> }

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct Heartbeat { pub local_user_id: i32, pub client_time: Option<DateTime<Utc>> }

#[derive(Message, Clone, Debug)]
#[rtype(result = "bool")]
pub struct IsUserOnline {pub room_id: ChatRoomId, pub local_user_id: LocalUserId }

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
            local_online: HashSet::new(),
        }
    }

    #[inline]
    fn touch(&mut self, local_user_id: LocalUserId) -> DateTime<Utc> {
        let now = Utc::now();
        if let Some(client) = &self.redis {
            let ttl = self.heartbeat_ttl.as_secs() as usize;
            let seen_key = format!("presence:user:{}:last_seen", local_user_id.0);
            let mut client = client.clone();
            actix::spawn(async move {
                // update last_seen with TTL
                let _ = client
                    .set_value_with_expiry(
                        &seen_key,
                        now.to_rfc3339(),
                        ttl,
                    )
                    .await;
            });
        }
        now
    }

    #[inline]
    fn mark_online(&mut self, room_id: ChatRoomId, local_user_id: LocalUserId) {
        //tracing::debug!(local_user_id, "presence: mark_online");
        if let Some(client) = &self.redis {
            let ttl = self.heartbeat_ttl.as_secs() as usize;
            let online_key = format!("presence:user:{}-{}:online", room_id.clone(), local_user_id.0);
            let seen_key = format!("presence:user:{}-{}:last_seen",room_id.0, local_user_id.0);
            let now = Utc::now();
            let mut client = client.clone();
            actix::spawn(async move {
                // refresh online flag and last_seen (both with TTL)
                let _ = client
                    .set_value_with_expiry(&online_key, true, ttl)
                    .await;
                let _ = client
                    .set_value_with_expiry(&seen_key, now.to_rfc3339(), ttl)
                    .await;
            });
        }
    }

    #[inline]
    fn mark_offline(&mut self, room_id: ChatRoomId, local_user_id: LocalUserId) {
        self.local_online.remove(&local_user_id.0);
        //tracing::debug!(local_user_id, "presence: mark_offline");
        if let Some(client) = &self.redis {
            let online_key = format!("presence:user:{}-{}:online",room_id.clone(), local_user_id.0);
            let seen_key = format!("presence:user:{}-{}:last_seen",room_id, local_user_id.0);
            let client = client.clone();
            let mut client = client; // owned clone above
            actix::spawn(async move {
                // best-effort delete both keys
                let _ = client.delete_key(&online_key).await; // ignore if missing
                let _ = client.delete_key(&seen_key).await;
            });
        }
    }

    /// Sweep users whose last_seen exceeded heartbeat_ttl and emit OnlineStopped
    pub fn sweep_timeouts(&mut self) {
        let Some(client) = &self.redis else { return; };
        let ttl = self.heartbeat_ttl;
        let client = client.clone();
        actix::spawn(async move {
            // We keep a simple list key of online users for snapshotting
            let list_key = "presence:online:users".to_string();
            // Read list (Vec<i32>), default empty
            let mut client = client.clone();
            let users: Vec<i32> = match client.get_value::<Vec<i32>>(&list_key).await {
                Ok(Some(v)) => v,
                _ => Vec::new(),
            };

            // Filter users that still have their online key alive
            let mut still_online: Vec<i32> = Vec::with_capacity(users.len());
            for uid in users {
                let ok = match client.get_value::<bool>(&format!("presence:user:{}:online", uid)).await {
                    Ok(Some(true)) => true,
                    _ => false,
                };
                if ok { still_online.push(uid); }
            }

            // Write back pruned list with a modest TTL so it refreshes regularly
            let _ = client
                .set_value_with_expiry(&list_key, still_online, ttl.as_secs() as usize)
                .await;
        });
    }
}
impl Actor for PresenceManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Schedule periodic sweep for timeouts; choose a sensible cadence (half of TTL, min 5s)
        let ttl_secs = self.heartbeat_ttl.as_secs();
        let sweep_every = if ttl_secs >= 10 { StdDuration::from_secs(ttl_secs / 2) } else { StdDuration::from_secs(5) };
        ctx.run_interval(sweep_every, |act, _ctx| {
            act.sweep_timeouts();
        });

        tracing::info!(
            ttl = ttl_secs,
            every = sweep_every.as_secs(),
            has_redis = self.redis.is_some(),
            "PresenceManager started"
        );
    }
}

impl Handler<OnlineJoin> for PresenceManager {
    type Result = ();
    fn handle(&mut self, msg: OnlineJoin, _ctx: &mut Context<Self>) -> Self::Result {
        // Local idempotency guard: `insert` returns false if already present
        // let already_local = !self.local_online.insert(msg.local_user_id.0);

        // Make OnlineJoin idempotent to avoid duplicate INFO logs
        if let Some(client) = &self.redis {
            let ttl = self.heartbeat_ttl.as_secs() as usize;
            let online_key = format!("presence:user:{}-{}:online",&msg.room_id, msg.local_user_id.0);
            let seen_key = format!("presence:user:{}-{}:last_seen",msg.room_id, msg.local_user_id.0);
            let mut client = client.clone();
            let started_at = msg.started_at;
            let user_id = msg.local_user_id;
            actix::spawn(async move {
                // let already_local = already_local; // captured from outer scope
                // // Check previous online flag BEFORE setting to avoid duplicate INFO logs
                // let was_online = matches!(client.get_value::<bool>(&online_key).await, Ok(Some(true)));

                // Refresh online flag and last_seen with TTL (idempotent write)
                let _ = client.set_value_with_expiry(&online_key, true, ttl).await;
                let _ = client.set_value_with_expiry(&seen_key, started_at.to_rfc3339(), ttl).await;

                // Maintain simple online list (best-effort)
                let list_key = "presence:online:users".to_string();
                let mut list: Vec<i32> = match client.get_value::<Vec<i32>>(&list_key).await {
                    Ok(Some(v)) => v,
                    _ => Vec::new(),
                };
                if !list.contains(&user_id.0) { list.push(user_id.0); }
                let _ = client.set_value_with_expiry(&list_key, list, ttl).await;

                // if already_local || was_online {
                //     tracing::debug!(local_user_id = user_id, ts = %started_at, "presence: online_join (duplicate)");
                // } else {
                //     tracing::info!(local_user_id = user_id, ts = %started_at, "presence: online_join");
                // }
            });
            return;
        }

        // Fallback (no Redis): best-effort mark online & touch, then INFO log once
        self.mark_online(msg.room_id, msg.local_user_id);
        self.touch(msg.local_user_id);
        // tracing::info!(local_user_id = msg.local_user_id, ts = %msg.started_at, "presence: online_join");
    }
}

impl Handler<OnlineLeave> for PresenceManager {
    type Result = ();
    fn handle(&mut self, msg: OnlineLeave, _ctx: &mut Context<Self>) -> Self::Result {
        self.mark_offline(msg.room_id, msg.local_user_id);
        // tracing::info!(local_user_id = msg.local_user_id, ts = %msg.left_at, "presence: online_leave");
    }
}

impl Handler<OnlineStopped> for PresenceManager {
    type Result = ();
    fn handle(&mut self, msg: OnlineStopped, _ctx: &mut Context<Self>) -> Self::Result {
        self.mark_offline(msg.room_id, msg.local_user_id);
        // tracing::info!(local_user_id = msg.local_user_id, ts = %msg.stopped_at, "presence: online_stopped(event)");
    }
}

impl Handler<Heartbeat> for PresenceManager {
    type Result = ();
    fn handle(&mut self, msg: Heartbeat, _ctx: &mut Context<Self>) -> Self::Result {
        let now = self.touch(LocalUserId(msg.local_user_id));
        // Decide online by Redis score vs ttl
        if let Some(client) = &self.redis {
            let client = client.clone();
            let ttl = self.heartbeat_ttl;
            let local_user_id = msg.local_user_id;
            actix::spawn(async move {
                let ttl_secs = ttl.as_secs() as usize;
                let mut client = client;
                // refresh keys
                let _ = client
                    .set_value_with_expiry(
                        &format!("presence:user:{}:online", local_user_id),
                        true,
                        ttl_secs,
                    )
                    .await;
                let _ = client
                    .set_value_with_expiry(
                        &format!("presence:user:{}:last_seen", local_user_id),
                        now.to_rfc3339(),
                        ttl_secs,
                    )
                    .await;
                // ensure user in the simple online list
                let list_key = "presence:online:users".to_string();
                let mut list: Vec<i32> = match client.get_value::<Vec<i32>>(&list_key).await {
                    Ok(Some(v)) => v,
                    _ => Vec::new(),
                };
                if !list.contains(&local_user_id) { list.push(local_user_id); }
                let _ = client
                    .set_value_with_expiry(&list_key, list, ttl_secs)
                    .await;
            });
        }
    }
}

impl Handler<IsUserOnline> for PresenceManager {
    type Result = ResponseFuture<bool>;
    fn handle(&mut self, msg: IsUserOnline, _ctx: &mut Context<Self>) -> Self::Result {
        let client = self.redis.clone();
        Box::pin(async move {
            if let Some(client) = client {
                let mut client = client; // RedisClient
                let key = format!("presence:user:{}-{}:online", msg.room_id, msg.local_user_id.0);
                if let Ok(Some(flag)) = client.get_value::<bool>(&key).await {
                    if flag { return true; }
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
            if let Some(client) = client {
                let mut client = client; // RedisClient
                let list_key = "presence:online:users".to_string();
                if let Ok(Some(ids)) = client.get_value::<Vec<i32>>(&list_key).await {
                    return ids.len();
                }
            }
            0
        })
    }
}