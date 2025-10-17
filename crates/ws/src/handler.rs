use crate::broker::phoenix_manager::{FetchHistoryDirect, GetLastRead, PhoenixManager};
use crate::broker::presence_manager::IsUserOnline;
use crate::phoenix_session::PhoenixSession;
use actix::Addr;
use actix_web::{
    web,
    web::{Data, Query},
    Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws;
use lemmy_db_views_chat::api::{HistoryQuery, LastReadQuery, PeerReadQuery};
use lemmy_db_views_local_user::LocalUserView;

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
pub async fn get_peer_status(
  phoenix: Data<Addr<PhoenixManager>>,
  q: Query<PeerReadQuery>,
) -> actix_web::Result<HttpResponse> {
  let room_id = q.room_id.clone().into();
  let online = phoenix
    .send(IsUserOnline {
      local_user_id: q.peer_id,
      room_id,
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
  stream: web::Payload,
  local_user_view: LocalUserView,
) -> Result<HttpResponse, Error> {
  let shared_key = local_user_view.person.shared_key;
  let ph_session = PhoenixSession::new(shared_key);
  ws::start(ph_session, &req, stream)
}
