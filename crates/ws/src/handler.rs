use crate::broker::phoenix_manager::{FetchHistoryDirect, GetLastRead, PhoenixManager};
use crate::broker::presence_manager::{IsUserOnline, PresenceManager};
use crate::phoenix_session::PhoenixSession;
use actix::Addr;
use actix_web::{
  web,
  web::{Data, Query},
  Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws;
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::local_user_view_from_jwt;
use lemmy_db_schema::newtypes::LocalUserId;
use lemmy_db_views_chat::api::{HistoryQuery, JoinRoomQuery, LastReadQuery, PeerReadQuery};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobError, FastJobErrorType};

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
  presence: Data<Addr<PresenceManager>>, // query presence actor instead of phoenix
  q: Query<PeerReadQuery>,
) -> actix_web::Result<HttpResponse> {
  let room_id = q.room_id.clone().into();
  let online = presence
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
  query: Query<JoinRoomQuery>,
  stream: web::Payload,
  phoenix: Data<Addr<PhoenixManager>>,
  presence: Data<Addr<PresenceManager>>,
  context: Data<FastJobContext>,
) -> Result<HttpResponse, Error> {
  // Extract query parameters similar to chat_ws
  let auth_token = query.token.clone();

  // Always initialize as Option<LocalUserId>
  let _local_user_id: Option<LocalUserId> = if let Some(jwt_token) = auth_token {
    match local_user_view_from_jwt(&jwt_token, &context).await {
      Ok((local_user, _session)) => Some(local_user.local_user.id),
      Err(_) => {
        return Err(Error::from(FastJobError::from(
          FastJobErrorType::IncorrectLogin,
        )));
      }
    }
  } else {
    None
  };

  let ph_session = PhoenixSession::new(
    phoenix.get_ref().clone(),
    presence.get_ref().clone(),
  );
  ws::start(ph_session, &req, stream)
}
