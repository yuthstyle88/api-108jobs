use crate::{broker::PhoenixManager, session::WsSession};
use actix::Addr;
use actix_web::{web::{Data, Payload}, Error, HttpRequest, Responder};
use actix_web::web::{Json, Query};
use actix_web_actors::ws;
use serde::Deserialize;
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::local_user_view_from_jwt;
use lemmy_db_schema::newtypes::ClientKey;
use crate::message::RegisterClientKeyMsg;
#[derive(Debug, Deserialize)]
pub struct JoinRoomQuery {
  pub token: Option<String>,
}

pub async fn chat_ws(
  req: HttpRequest,
  query: Query<JoinRoomQuery>,
  stream: Payload,
  phoenix: Data<Addr<PhoenixManager>>,
  context: Data<FastJobContext>,
) ->  Result<impl Responder, Error>  {
  let token = query.token.clone();
  let mut client_key = None;
  let mut user_id = None;

  if let Some(jwt) = token {
    let local_user= local_user_view_from_jwt(&jwt, &context).await?;
    user_id = Some(local_user.local_user.id);
    if let Some(public_key) = local_user.local_user.public_key {
      client_key = create_client_key(&public_key);
    }
  }

  let session = WsSession {
    id: uuid::Uuid::new_v4().to_string(),
    phoenix_manager: phoenix.get_ref().clone(),
    client_key: RegisterClientKeyMsg { user_id, client_key },
  };
  ws::start(session, &req, stream)
}

fn create_client_key(value: &str) -> Option<ClientKey> {
  webcryptobox::import_public_key_pem(value.as_bytes())
   .ok()
   .map(ClientKey)
}

