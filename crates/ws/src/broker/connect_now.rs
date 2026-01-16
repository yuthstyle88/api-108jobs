use crate::protocol::phx_helper::connect;
use crate::broker::init_socket::InitSocket;
use crate::broker::manager::PhoenixManager;
use actix::{AsyncContext, Context, Handler, Message};

#[derive(Message)]
#[rtype(result = "()")]
pub struct ConnectNow;

impl Handler<ConnectNow> for PhoenixManager {
  type Result = ();

  fn handle(&mut self, _msg: ConnectNow, ctx: &mut Context<Self>) -> Self::Result {
    let socket = self.socket.clone();
    let addr = ctx.address();
    actix::spawn(async move {
      match connect(socket).await {
        Ok(sock) => {
          addr.do_send(InitSocket(sock));
        }
        Err(e) => {
          tracing::error!("Failed to connect Phoenix socket: {}", e);
        }
      }
    });
  }
}
