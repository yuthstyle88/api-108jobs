use super::WsState;
use crate::proxy::PhoenixProxy;
use crate::session::Session;
use actix_ws::Message;
use std::sync::Arc;
use lemmy_api_utils::context::FastJobContext;
use lemmy_utils::error::FastJobResult;

#[derive(Clone)]
pub struct Connected {
    pub session: Session,
}

impl WsState for Connected {
    type Next = Self;

    async fn handle(
        self,
        _session: Session,
        _ctx: Arc<FastJobContext>,
        proxy: Arc<PhoenixProxy>,
        msg: Message,
    ) -> FastJobResult<(Self, Option<Message>)> {
        if let Message::Text(text) = &msg {
            proxy
                .send_message_to_phoenix("room:debug".to_string(), text.clone().parse()?)
                .await;
        }

        Ok((self, None))
    }

    fn name(&self) -> &'static str {
        "Connected"
    }
}
