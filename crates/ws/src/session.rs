use actix_ws::Session as WsSession;
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Wrapped Actix WebSocket session for shared access
#[derive(Clone)]
pub struct Session {
  inner: Arc<RwLock<WsSession>>,
}

impl Session {
  pub fn new(session: WsSession) -> Self {
    Self {
      inner: Arc::new(RwLock::new(session)),
    }
  }

  pub async fn text(&self, text: String) -> FastJobResult<()> {
    let mut session = self.inner.write().await;
    session
      .text(text)
      .await
      .with_fastjob_type(FastJobErrorType::InvalidBodyField)
  }
}

impl PartialEq for Session {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.inner, &other.inner)
  }
}
