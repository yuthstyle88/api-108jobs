use crate::{broker::PhoenixManager, session::WsSession};
use actix::Addr;
use actix_web::{web::{Data, Payload}, Error, HttpRequest, Responder};
use actix_web::web::Query;
use actix_web_actors::ws;
use serde::Deserialize;
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::local_user_view_from_jwt;
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
  let mut client_key = None;
  let mut user_id = None;

  if let Some(jwt) = token {
    let local_user= local_user_view_from_jwt(&jwt, &context).await;
    match local_user {
      Ok(local_user) => {
        user_id = Some(local_user.local_user.id);
        if let Some(public_key) = local_user.local_user.public_key {
          client_key = create_client_key(&public_key);
        }
      }
      Err(_) => {
        eprintln!("Failed to get local user from jwt");
      }
    }
  }

  let session = WsSession {
    phoenix_manager: phoenix.get_ref().clone(),
    client_msg: RegisterClientMsg { user_id, client_key, room_id, room_name },
  };
  ws::start(session, &req, stream)
}

fn create_client_key(value: &str) -> Option<String> {
  webcryptobox::import_public_key_pem(value.as_bytes())
   .ok()
   .and_then(|key| webcryptobox::export_public_key_pem(&key).ok())
   .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

