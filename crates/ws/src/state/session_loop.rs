use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::task::LocalSet;
use actix_ws::Message;
use futures_util::StreamExt;
use lemmy_api_utils::context::FastJobContext;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use crate::proxy::PhoenixProxy;
use crate::session::Session;
use crate::state::WsState;

pub fn run_session_loop<S>(
  state: S,
  session: Session,
  ctx: Arc<FastJobContext>,
  proxy: Arc<PhoenixProxy>,
  mut stream: impl StreamExt<Item = Result<Message, actix_ws::ProtocolError>> + Unpin + 'static,
) -> Pin<Box<dyn Future<Output = FastJobResult<()>>>>
where
    S: WsState + 'static,
    S::Next: WsState + 'static,
{
  Box::pin(async move {
    let local = LocalSet::new();
    local.spawn_local(async move {
      if let Some(Ok(msg)) = stream.next().await {
        let (next_state, reply) = state
            .handle(session.clone(), ctx.clone(), proxy.clone(), msg)
            .await?;

        if let Some(Message::Text(text)) = reply {
          let text_str = String::from_utf8(Vec::from(text.into_bytes()))
              .with_fastjob_type(FastJobErrorType::InvalidBodyField)?;
          session.text(text_str).await?;
        }

        run_session_loop::<S::Next>(next_state, session, ctx, proxy, stream).await?;
      }
      Ok::<(), lemmy_utils::error::FastJobError>(())
    });
    Ok(local.await)
  })
}