use crate::broker::manager::{FetchHistoryDirect, GetLastRead, GetUnreadSnapshot, PhoenixManager};
use crate::server::session::PhoenixSession;
use crate::presence::IsUserOnline;
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
  println!("get_peer_status: online={:?}", online);
  Ok(HttpResponse::Ok().json(serde_json::json!({
    "online": online,
  })))
}

pub async fn phoenix_ws(
  req: HttpRequest,
  query: Query<JoinRoomQuery>,
  stream: web::Payload,
  context: Data<FastJobContext>,
) -> Result<HttpResponse, Error> {
  // Extract query parameters similar to chat_ws
  let auth_token = query.token.clone();

  let (shared_key, local_user_id): (Option<String>, Option<LocalUserId>) = if let Some(jwt_token) = auth_token {
    match local_user_view_from_jwt(&jwt_token, &context).await {
      Ok((local_user_view, _session)) => (local_user_view.person.shared_key, Some(local_user_view.local_user.id)),
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
