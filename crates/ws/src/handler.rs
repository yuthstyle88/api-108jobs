use crate::session::Session;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_ws::handle;
use lemmy_api_utils::context::FastJobContext;

use std::sync::Arc;
use crate::proxy::PhoenixProxy;
use crate::state::resolving_room::ResolvingRoom;
use crate::state::session_loop::run_session_loop;

pub async fn websocket_handler(
    req: HttpRequest,
    body: web::Payload,
    proxy: web::Data<Arc<PhoenixProxy>>,
    context: web::Data<Arc<FastJobContext>>,
) -> impl Responder {
    let (res, conn, msg_stream) = match handle(&req, body) {
        Ok(parts) => parts,
        Err(err) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("WebSocket error: {}", err)
            }))
        }
    };

    let session = Session::new(conn);

    let initial_state = ResolvingRoom;

    actix_rt::spawn(run_session_loop(
        initial_state,
        session,
        context.get_ref().clone(),
        proxy.get_ref().clone(),
        msg_stream,
    ));

    res
}
