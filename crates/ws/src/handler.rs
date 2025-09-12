use crate::message::RegisterClientMsg;
use crate::{broker::{FetchHistoryDirect, PhoenixManager}, session::WsSession};
use actix::Addr;
use actix_web::{web::{Data, Payload, Query}, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::local_user_view_from_jwt;
use lemmy_db_schema::newtypes::{ChatRoomId, PaginationCursor};
use lemmy_utils::error::{FastJobError, FastJobErrorType};
use serde::Deserialize;
use lemmy_db_views_local_user::LocalUserView;

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
      user_id: local_user_view.local_user.id,
      room_id: q.room_id.clone(),
      page_cursor: q.cursor.clone(),
      limit: q.limit.or(Some(20)),
      page_back: q.back,
    })
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))??;

  Ok(HttpResponse::Ok().json(resp))
}

#[derive(Debug, Deserialize)]
pub struct JoinRoomQuery {
  pub token: Option<String>,
  #[serde(alias = "roomId", alias = "room_id")]
  pub room_id: String,
  #[serde(alias = "roomName", alias = "room_name")]
  pub room_name: Option<String>,
  #[serde(alias = "userId", alias = "user_id")]
  pub user_id: Option<i32>,
}

pub async fn chat_ws(
  req: HttpRequest,
  query: Query<JoinRoomQuery>,
  stream: Payload,
  phoenix: Data<Addr<PhoenixManager>>,
  context: Data<FastJobContext>,
) -> Result<impl Responder, Error> {
  // Extract query parameters
  let auth_token = query.token.clone();
  let room_id = query.room_id.clone().into();
  let room_name = query
    .room_name
    .clone()
    .unwrap_or_else(|| query.room_id.clone())
    .into();

  // Initialize authentication data
  let mut user_id = None;

  // Handle authentication if token exists
  if let Some(jwt_token) = auth_token {
    match local_user_view_from_jwt(&jwt_token, &context).await {
      Ok((local_user, _session)) => {
        user_id = Some(local_user.local_user.id);
      }
      Err(_) => {
        return Err(Error::from(FastJobError::from(FastJobErrorType::IncorrectLogin)));
      }
    }
  }

  // Fallback: if user_id is still None, use query.user_id (if supplied)
  if user_id.is_none() {
    if let Some(uid) = query.user_id {
      use lemmy_db_schema::newtypes::LocalUserId;
      user_id = Some(LocalUserId(uid));
    }
  }

  // Create websocket session
  let ws_session = WsSession::new(
    phoenix.get_ref().clone(),
    RegisterClientMsg {
      user_id,
      room_id,
      room_name,
    }
  );

  // Start websocket connection
  ws::start(ws_session, &req, stream)
}