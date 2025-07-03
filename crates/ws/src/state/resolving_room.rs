use super::WsState;
use crate::proxy::PhoenixProxy;
use crate::session::Session;
use crate::state::connected::Connected;
use actix_ws::Message;
use std::sync::Arc;
use lemmy_api_utils::context::FastJobContext;
use lemmy_utils::error::FastJobResult;

#[derive(Debug, Clone)]
pub struct ResolvingRoom;

#[async_trait::async_trait]
impl WsState for ResolvingRoom {
    type Next = Connected;

    async fn handle(
        self,
        session: Session,
        _ctx: Arc<FastJobContext>,
        _proxy: Arc<PhoenixProxy>,
        _msg: Message,
    ) -> FastJobResult<(Connected, Option<Message>)> {
        // Simulate resolving logic
        Ok((
            Connected { session },
            Some(Message::Text("joined room".into())),
        ))
    }

    fn name(&self) -> &'static str {
        "ResolvingRoom"
    }
}
