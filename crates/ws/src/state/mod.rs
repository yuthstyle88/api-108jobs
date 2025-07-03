use std::future::Future;
use crate::proxy::PhoenixProxy;
use crate::session::Session;

use std::sync::Arc;
use actix_ws::Message;
use lemmy_api_utils::context::FastJobContext;
use lemmy_utils::error::FastJobResult;

/// WebSocket session state machine trait

pub trait WsState: Send + Sync {
    type Next: WsState;

    fn handle(
        self,
        session: Session,
        ctx: Arc<FastJobContext>,
        proxy: Arc<PhoenixProxy>,
        msg: Message,
    ) ->  impl Future<Output = FastJobResult<(Self::Next, Option<Message>)>> + Send;

    fn name(&self) -> &'static str;
}

pub mod session_loop;
pub mod resolving_room;
pub mod connected;