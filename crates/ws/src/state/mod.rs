use crate::proxy::PhoenixProxy;
use crate::session::Session;

use std::sync::Arc;
use actix_ws::Message;
use lemmy_api_utils::context::FastJobContext;
use lemmy_utils::error::FastJobResult;

use async_trait::async_trait;

/// WebSocket session state machine trait
#[async_trait]
pub trait WsState: Send + Sync {
    type Next: WsState;

    async fn handle(
        self,
        session: Session,
        ctx: Arc<FastJobContext>,
        proxy: Arc<PhoenixProxy>,
        msg: Message,
    ) -> FastJobResult<(Self::Next, Option<Message>)>;

    fn name(&self) -> &'static str;
}

pub mod session_loop;
pub mod resolving_room;
pub mod connected;