use crate::broker::phoenix_manager::PhoenixManager;
use actix::{Context, Handler, Message};
use phoenix_channels_client::Socket;
use std::sync::Arc;

#[derive(Message)]
#[rtype(result = "()")]
pub struct InitSocket(pub Arc<Socket>);

impl Handler<InitSocket> for PhoenixManager {
  type Result = ();

  fn handle(&mut self, msg: InitSocket, _ctx: &mut Context<Self>) {
    self.socket = msg.0;
    tracing::info!("Connect status: {:?}", self.socket.status());
  }
}
