use crate::{broker::PhoenixManager, session::WsSession};
use actix::Addr;
use actix_web::{web::{Data, Payload}, Error, HttpRequest, Responder};
use actix_web::web::Query;
use actix_web_actors::ws;
use serde::Deserialize;
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::local_user_view_from_jwt;
use lemmy_db_schema::newtypes::ClientKey;
use lemmy_utils::error::FastJobResult;
use lemmy_utils::crypto::{Crypto, DataBuffer};
use crate::message::RegisterClientMsg;

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
) ->  Result<impl Responder, Error>  {
  let token = query.token.clone();
  let room_id = query.room_id.clone().into();
  let room_name = query.room_name.clone().into();
  let mut client_key = "".to_string();
  let mut user_id = None;
  let mut session_id = "".to_string();
  if let Some(jwt) = token {
    let local_user= local_user_view_from_jwt(&jwt, &context).await;
    match local_user {
      Ok((local_user, session)) => {
        session_id = session;
        user_id = Some(local_user.local_user.id);
        if let Some(public_key) = local_user.local_user.public_key {
          let public_key = hex::decode(&public_key).map_err(|_| actix_web::error::ErrorBadRequest("Failed to decode public key"))?;
          client_key =  create_client_key(&public_key)?.0;
        }
      }
      Err(_) => {
        eprintln!("Failed to get local user from jwt");
      }
    }
  }

  let session = WsSession::new(
    phoenix.get_ref().clone(),
    RegisterClientMsg { user_id, room_id, room_name },
    session_id,
    client_key
  );

  ws::start(session, &req, stream)
}

fn create_client_key(value: &[u8]) -> FastJobResult<ClientKey> {
  let public_key = DataBuffer::from_vec(value);
  let pem = Crypto::import_public_key(public_key)?;
  let pem = hex::encode(&pem);
  Ok(ClientKey(pem))
}


