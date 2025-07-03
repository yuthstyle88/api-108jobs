use actix_web::{web, Error, HttpRequest, Responder};
use actix_web_actors::ws;
use lemmy_api_utils::context::FastJobContext;
use crate::session::WsChatSession;

pub async fn chat_ws(req: HttpRequest, stream: web::Payload, context: web::Data<FastJobContext>) -> Result<impl Responder, Error> {
    let session = WsChatSession::new(context.clone());
    ws::start(session, &req, stream)
}