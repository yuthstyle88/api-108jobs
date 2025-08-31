use crate::message::RegisterClientMsg;
use crate::{broker::PhoenixManager, session::WsSession};
use actix::Addr;
use actix_web::web::Query;
use actix_web::{
  web::{Data, Payload},
  Error, HttpRequest, Responder,
};
use actix_web_actors::ws;
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::local_user_view_from_jwt;
use serde::Deserialize;
use lemmy_utils::error::{FastJobError, FastJobErrorType};

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
  let mut shared_key = "".to_string();
  let mut user_id = None;
  let mut session_id = "".to_string();

  // Handle authentication if token exists
  if let Some(jwt_token) = auth_token {
    match local_user_view_from_jwt(&jwt_token, &context).await {
      Ok((local_user, _session)) => {
        // Align IV derivation with frontend: use the JWT token as sessionId
        // Frontend encrypts with AES-CBC using IV derived from the JWT string.
        // Using the same here ensures decrypt/encrypt symmetry.
        session_id = jwt_token.clone();
        user_id = Some(local_user.local_user.id);
        // Get encryption key if user has public key (this actually stores the shared secret in DB)
        shared_key = local_user.person.public_key
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
    },
    session_id,
    shared_key,
  );

  // Start websocket connection
  ws::start(ws_session, &req, stream)
}