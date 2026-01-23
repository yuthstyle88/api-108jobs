use crate::broker::manager::{
  FetchHistoryDirect, GetLastRead, GetPresenceSnapshot, GetUnreadSnapshot, PhoenixManager,
};
use crate::presence::{IsUserOnline, PresenceManager};
use crate::server::session::PhoenixSession;
use actix::prelude::*;
use actix::ActorContext;
use actix::Addr;
use actix_web::{
  web,
  web::{Data, Query},
  Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_api_utils::utils::local_user_view_from_jwt;
use app_108jobs_db_schema::newtypes::LocalUserId;
use app_108jobs_db_views_chat::api::{HistoryQuery, JoinRoomQuery, LastReadQuery, PeerReadQuery};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_utils::error::{FastJobError, FastJobErrorType};
use tokio::time::Duration;

/// Direct history API: query DB without routing through chat/broker
pub async fn get_history(
  phoenix: Data<Addr<PhoenixManager>>,
  q: Query<HistoryQuery>,
  local_user_view: LocalUserView,
) -> actix_web::Result<HttpResponse> {
  let resp = phoenix
    .send(FetchHistoryDirect {
      local_user_id: local_user_view.local_user.id,
      room_id: q.room_id.clone(),
      page_cursor: q.cursor.clone(),
      limit: q.limit.or(Some(20)),
      page_back: q.back,
    })
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))??;

  Ok(HttpResponse::Ok().json(resp))
}

pub async fn get_last_read(
  phoenix: Data<Addr<PhoenixManager>>,
  q: Query<LastReadQuery>,
  _local_user_view: LocalUserView,
) -> actix_web::Result<HttpResponse> {
  let resp = phoenix
    .send(GetLastRead {
      local_user_id: q.peer_id.clone(),
      room_id: q.room_id.clone(),
    })
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))??;

  Ok(HttpResponse::Ok().json(resp))
}

/// Fetch unread snapshot for the current user across all rooms.
/// This replaces the previous WS-based initial unread emission.
pub async fn get_unread_snapshot(
  phoenix: Data<Addr<PhoenixManager>>,
  local_user_view: LocalUserView,
) -> actix_web::Result<HttpResponse> {
  let resp = phoenix
    .send(GetUnreadSnapshot {
      local_user_id: local_user_view.local_user.id,
    })
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))??;

  Ok(HttpResponse::Ok().json(resp))
}
pub async fn get_peer_status(
  phoenix: Data<Addr<PhoenixManager>>,
  q: Query<PeerReadQuery>,
) -> actix_web::Result<HttpResponse> {
  let online = phoenix
    .send(IsUserOnline {
      local_user_id: q.peer_id,
    })
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
  Ok(HttpResponse::Ok().json(serde_json::json!({
    "online": online,
  })))
}

pub async fn get_presence_snapshot(
  presence: Data<Addr<PresenceManager>>,
  local_user_view: LocalUserView,
) -> actix_web::Result<HttpResponse> {
  let resp = presence
    .send(GetPresenceSnapshot {
      local_user_id: local_user_view.local_user.id,
    })
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))??;

  Ok(HttpResponse::Ok().json(resp))
}

pub async fn phoenix_ws(
  req: HttpRequest,
  query: Query<JoinRoomQuery>,
  stream: web::Payload,
  context: Data<FastJobContext>,
) -> Result<HttpResponse, Error> {
  // Extract query parameters similar to chat_ws
  let auth_token = query.token.clone();

  let (shared_key, local_user_id): (Option<String>, Option<LocalUserId>) =
    if let Some(jwt_token) = auth_token {
      match local_user_view_from_jwt(&jwt_token, &context).await {
        Ok((local_user_view, _session)) => (
          local_user_view.person.shared_key,
          Some(local_user_view.local_user.id),
        ),
        Err(_) => {
          return Err(Error::from(FastJobError::from(
            FastJobErrorType::IncorrectLogin,
          )));
        }
      }
    } else {
      (None, None)
    };
  let ph_session = PhoenixSession::new(shared_key, local_user_id);
  ws::start(ph_session, &req, stream)
}

// ============ Delivery location WS (employer/rider viewer) ============

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TokenQuery {
  pub token: Option<String>,
}

/// WebSocket that streams delivery location updates for a given postId.
/// Auth: JWT in `?token=` query; must resolve to a valid user. Authorization to
/// a specific delivery should be enforced at a higher layer when assignment flow is implemented.
pub async fn delivery_location_ws(
  req: HttpRequest,
  path: web::Path<i32>,
  query: Query<TokenQuery>,
  stream: web::Payload,
  context: Data<FastJobContext>,
) -> Result<HttpResponse, Error> {
  // Authenticate via token (same pattern as phoenix_ws)
  if let Some(jwt_token) = &query.token {
    // If invalid, return 401
    if local_user_view_from_jwt(jwt_token, &context).await.is_err() {
      return Err(Error::from(FastJobError::from(
        FastJobErrorType::IncorrectLogin,
      )));
    }
  } else {
    return Err(Error::from(FastJobError::from(
      FastJobErrorType::IncorrectLogin,
    )));
  }

  let post_id = *path;
  let session = DeliveryLocationSession::new(post_id, context.get_ref().clone());
  ws::start(session, &req, stream)
}

struct DeliveryLocationSession {
  post_id: i32,
  ctx: FastJobContext,
  last_sent: Option<String>,
}

impl DeliveryLocationSession {
  fn new(post_id: i32, ctx: FastJobContext) -> Self {
    Self {
      post_id,
      ctx,
      last_sent: None,
    }
  }
}

impl Actor for DeliveryLocationSession {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    // Emit last known + start a lightweight polling loop (imitate existing utils style)
    let key = format!("delivery:current:{}", self.post_id);
    let mut redis = self.ctx.redis().clone();
    let addr = ctx.address();
    actix::spawn(async move {
      // One-time fetch of last known
      if let Ok(Some::<serde_json::Value>(val)) = redis.get_value(&key).await {
        if let Ok(text) = serde_json::to_string(&val) {
          addr.do_send(EmitRaw(text));
        }
      }
    });

    // Poll every 2 seconds for changes (can be replaced with pub/sub helper later)
    let mut redis = self.ctx.redis().clone();
    let post_id = self.post_id;
    let addr = ctx.address();
    actix::spawn(async move {
      let mut tick = tokio::time::interval(Duration::from_secs(2));
      let mut last: Option<String> = None;
      loop {
        tick.tick().await;
        let key = format!("delivery:current:{}", post_id);
        if let Ok(Some::<serde_json::Value>(val)) = redis.get_value(&key).await {
          if let Ok(text) = serde_json::to_string(&val) {
            if last.as_deref() != Some(&text) {
              last = Some(text.clone());
              addr.do_send(EmitMaybe { payload: text });
            }
          }
        }
      }
    });
  }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
struct EmitRaw(String);

impl actix::Handler<EmitRaw> for DeliveryLocationSession {
  type Result = ();
  fn handle(&mut self, msg: EmitRaw, ctx: &mut Self::Context) -> Self::Result {
    ctx.text(msg.0);
  }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
struct EmitMaybe {
  payload: String,
}

impl actix::Handler<EmitMaybe> for DeliveryLocationSession {
  type Result = ();
  fn handle(&mut self, msg: EmitMaybe, ctx: &mut Self::Context) -> Self::Result {
    if self.last_sent.as_deref() != Some(&msg.payload) {
      self.last_sent = Some(msg.payload.clone());
      ctx.text(msg.payload);
    }
  }
}

impl actix::StreamHandler<Result<ws::Message, ws::ProtocolError>> for DeliveryLocationSession {
  fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match item {
      Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
      Ok(ws::Message::Pong(_)) => {}
      Ok(ws::Message::Text(_)) => {}
      Ok(ws::Message::Binary(_)) => {}
      Ok(ws::Message::Close(reason)) => {
        ctx.close(reason);
        ctx.stop();
      }
      Ok(ws::Message::Nop) => {}
      Err(_) => {
        ctx.stop();
      }
      _ => {}
    }
  }
}
