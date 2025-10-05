use std::collections::HashMap;
use crate::api::RegisterClientMsg;
use actix::Addr;
use actix_web::{web, web::{Data, Query}, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::local_user_view_from_jwt;
use lemmy_db_schema::newtypes::{ChatRoomId, PaginationCursor, LocalUserId};
use lemmy_utils::error::{FastJobError, FastJobErrorType};
use serde::Deserialize;
use lemmy_db_views_local_user::LocalUserView;
use crate::broker::phoenix_manager::{FetchHistoryDirect, PhoenixManager};
use crate::broker::presence_manager::PresenceManager;
use crate::phoenix_session::PhoenixSession;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryQuery {
  pub room_id: ChatRoomId,
  pub cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
  pub back: Option<bool>,
}

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

#[derive(Debug, Clone, Deserialize)]
pub struct JoinRoomQuery {
  /// Phoenix Socket(..., { params: { token } }) → ?token=...
  #[serde(default)]
  pub token: Option<String>,

  /// FE อาจไม่ส่ง room มาทาง query (จะได้จาก topic ตอน phx_join)
  #[serde(alias = "roomId", alias = "room_id", alias = "room", default)]
  pub room_id: Option<String>,

  #[serde(alias = "roomName", alias = "room_name", default)]
  pub room_name: Option<String>,

  #[serde(alias = "userId", alias = "user_id", default)]
  pub local_user_id: Option<i32>,

  /// เก็บพารามิเตอร์อื่น ๆ (เช่น vsn) ป้องกัน deserialize error
  #[serde(flatten)]
  pub extra: HashMap<String, String>,
}
impl JoinRoomQuery {
  /// คืน room id จาก query หรือ topic (เช่น "room:abc123")
  pub fn resolve_room_from_query_or_topic(&self, topic: Option<&str>) -> Option<String> {
    if let Some(r) = self.room_id.clone() { return Some(r); }
    if let Some(n) = self.room_name.clone() { return Some(n); }
    if let Some(t) = topic {
      if let Some(id) = t.strip_prefix("room:") {
        return Some(id.to_string());
      }
    }
    None
  }
}

pub async fn phoenix_ws(
  req: HttpRequest,
  query: Query<JoinRoomQuery>,
  stream: web::Payload,
  phoenix: Data<Addr<PhoenixManager>>,
  presence: Data<Addr<PresenceManager>>,
  context: Data<FastJobContext>,
) -> Result<HttpResponse, Error> {
  // Extract query parameters similar to chat_ws
  let auth_token = query.token.clone();
  let params = query.into_inner();

  // Always initialize as Option<LocalUserId>
  let local_user_id: Option<LocalUserId> = if let Some(jwt_token) = auth_token {
    match local_user_view_from_jwt(&jwt_token, &context).await {
      Ok((local_user, _session)) => Some(local_user.local_user.id),
      Err(_) => {
        return Err(Error::from(FastJobError::from(FastJobErrorType::IncorrectLogin)));
      }
    }
  } else {
    None
  };

  // Try to resolve initial room id from query (topic will refine it on phx_join)
  let initial_room = match params.resolve_room_from_query_or_topic(None) {
    Some(r) => ChatRoomId::from_channel_name(r.as_str())
      .unwrap_or_else(|_| ChatRoomId(r)),
    None => ChatRoomId(String::new()),
  };

  let ph_session = PhoenixSession::new(
    phoenix.get_ref().clone(),
    presence.get_ref().clone(),
    params,
    RegisterClientMsg {
      local_user_id,
      room_id: initial_room,
    }
  );
  ws::start(ph_session, &req, stream)
}