use std::collections::HashSet;
use crate::{broker::PhoenixManager, session::WsSession};
use actix::Addr;
use actix_web::{web, web::{Data, Payload}, Error, HttpRequest, Responder};
use actix_web_actors::ws;
use lemmy_api_utils::context::FastJobContext;
use lemmy_utils::crypto::Crypto;

pub async fn chat_ws(
  req: HttpRequest,
  stream: Payload,
  phoenix: Data<Addr<PhoenixManager>>,
  crypto: Data<Crypto>,
) -> Result<impl Responder, Error> {
  let crypto = crypto.get_ref().clone();
  let session = WsSession {
    crypto,
    id: uuid::Uuid::new_v4().to_string(),
    phoenix_manager: phoenix.get_ref().clone(),
  };
  ws::start(session, &req, stream)
}
