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
use lemmy_db_schema::newtypes::SharedSecret;
use lemmy_utils::crypto::{Crypto, DataBuffer};
use lemmy_utils::error::FastJobResult;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct JoinRoomQuery {
  pub token: Option<String>,
  pub room_id: String,
  pub room_name: String,
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
  let room_name = query.room_name.clone().into();

  // Initialize authentication data
  let mut shared_key = "".to_string();
  let mut user_id = None;
  let mut session_id = "".to_string();

  // Handle authentication if token exists
  if let Some(jwt_token) = auth_token {
    match local_user_view_from_jwt(&jwt_token, &context).await {
      Ok((local_user, session)) => {
        session_id = session;
        user_id = Some(local_user.local_user.id);
        // Get encryption key if user has public key
        if let Some(public_key) = local_user.local_user.public_key {
          shared_key = public_key;
        }
      }
      Err(_) => {
        eprintln!("Failed to get local user from jwt");
      }
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

fn create_client_key(value: &[u8]) -> FastJobResult<SharedSecret> {
  let public_key = DataBuffer::from_vec(value);
  let pem = Crypto::import_public_key(public_key)?;
  let pem = hex::encode(&pem);
  Ok(SharedSecret(pem))
}
