use std::collections::HashSet;
use crate::{broker::PhoenixManager, session::WsSession};
use actix::Addr;
use actix_web::{web, web::{Data, Payload}, Error, HttpRequest, Responder};
use actix_web_actors::ws;
use lemmy_api_utils::context::FastJobContext;

pub async fn chat_ws(
  req: HttpRequest,
  stream: Payload,
  phoenix: Data<Addr<PhoenixManager>>,
) -> Result<impl Responder, Error> {
  let session = WsSession {
    id: uuid::Uuid::new_v4().to_string(),
    phoenix_manager: phoenix.get_ref().clone(),
  };
  ws::start(session, &req, stream)
}

