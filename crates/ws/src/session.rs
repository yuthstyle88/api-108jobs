use std::collections::HashSet;
use crate::{
  bridge_message::{BridgeMessage, MessageSource},
  broker::{GetDbPool, PhoenixManager},
  message::StoreChatMessage,
};
use actix::{fut::wrap_future, Actor, ActorContext, Addr, AsyncContext, Handler, StreamHandler};
use actix_broker::BrokerSubscribe;
use actix_web_actors::ws;
use chrono::Utc;
use futures_util::FutureExt;
use lemmy_db_schema::{
  newtypes::{ChatRoomId, LocalUserId, PostId},
  source::chat_message::ChatMessageInsertForm,
  traits::Crud,
  utils::DbPool,
};
use lemmy_utils::error::FastJobError;
use serde::Deserialize;
use serde_json::json;

pub struct WsSession {
  pub(crate) id: String,
  pub(crate) phoenix_manager: Addr<PhoenixManager>,
  pub(crate) subscribed_channels: HashSet<String>,
}

impl Actor for WsSession {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    self.subscribe_system_sync::<BridgeMessage>(ctx);

    self.subscribed_channels = HashSet::new();

    log::info!("WebSocket session {} started", self.id);
  }
}

impl Handler<BridgeMessage> for WsSession {
  type Result = ();

  fn handle(&mut self, msg: BridgeMessage, ctx: &mut Self::Context) {
    if self.subscribed_channels.contains(&msg.channel) {
      if let Ok(text) = serde_json::to_string(&msg.payload) {
        ctx.text(text);
      }
    }
  }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum WsClientCommand {
  JoinRoom {
    sender_id: LocalUserId,
    receiver_id: LocalUserId,
    post_id: PostId,
  },
  SendMessage {
    sender_id: LocalUserId,
    room_id: ChatRoomId,
    content: String,
  },
  LeaveRoom {
    room_id: ChatRoomId,
  },
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
  fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match msg {
      Ok(ws::Message::Text(text)) => {
        if let Ok(cmd) = serde_json::from_str::<WsClientCommand>(&text) {
          match cmd {
            WsClientCommand::JoinRoom {
              sender_id,
              receiver_id,
              post_id,
            } => {
              let mut subscribed_channels = self.subscribed_channels.clone();
              let phoenix = self.phoenix_manager.clone();

              let fut = async move {
                use lemmy_db_schema::source::{
                  chat_room::{ChatRoom, ChatRoomInsertForm},
                  chat_room_member::{ChatRoomMember, ChatRoomMemberInsertForm},
                };

                let pool = phoenix.send(GetDbPool).await.unwrap();
                let mut db_pool = DbPool::Pool(&pool);
                let maybe_room =
                  ChatRoomMember::find_room_by_members(&mut db_pool, sender_id, receiver_id).await;

                let room_id = match maybe_room {
                  Ok(Some(existing)) => existing,
                  _ => {
                    let now = Utc::now();

                    // Step 1: Create a new chat room
                    let new_room = ChatRoom::create(
                      &mut db_pool,
                      &ChatRoomInsertForm {
                        post_id,
                        created_at: now,
                        updated_at: now,
                      },
                    )
                    .await?;
                    let room_id = new_room.id;

                    // Step 2: Create room members (sender + receiver)
                    for user_id in [sender_id, receiver_id] {
                      let member_form = ChatRoomMemberInsertForm { room_id, user_id };

                      if let Err(e) = ChatRoomMember::create(&mut db_pool, &member_form).await {
                        return Err(e);
                      }
                    }

                    room_id
                  }
                };

                subscribed_channels.insert(format!("room:{}", room_id));

                let msg = BridgeMessage {
                  source: MessageSource::WebSocket,
                  channel: format!("room:{}", room_id.0),
                  event: "phx_join".to_string(),
                  payload: json!({}),
                };

                phoenix.do_send(msg);
                Ok::<(), FastJobError>(())
              };
              ctx.spawn(wrap_future(fut.map(|res| {
                if let Err(e) = res {
                  eprintln!("Join room error: {e}");
                }
              })));
            }
            WsClientCommand::SendMessage {
              sender_id,
              room_id,
              content,
            } => {
              let phoenix_manager = self.phoenix_manager.clone();
              let now = Utc::now();

              let chat = ChatMessageInsertForm {
                room_id,
                sender_id,
                content,
                file_url: None,
                file_type: None,
                file_name: None,
                status: 1,
                created_at: now,
                updated_at: now,
              };

              let fut = async move {
                match phoenix_manager
                  .send(StoreChatMessage {
                    message: chat.clone(),
                  })
                  .await
                {
                  Ok(()) => {
                    if let Ok(payload) = serde_json::to_value(&chat) {
                      let msg = BridgeMessage {
                        source: MessageSource::WebSocket,
                        channel: format!("room:{}", room_id),
                        event: "new_msg".to_string(),
                        payload,
                      };
                      phoenix_manager.do_send(msg);
                    }
                    Ok(())
                  }
                  Err(e) => Err(e.into()),
                }
              };

              ctx.spawn(wrap_future(fut.map(|res: Result<(), FastJobError>| {
                if let Err(e) = res {
                  log::error!("WebSocket send message failed: {:?}", e);
                }
              })));
            }

            WsClientCommand::LeaveRoom { room_id } => {
              let msg = BridgeMessage {
                source: MessageSource::WebSocket,
                channel: format!("room:{}", room_id),
                event: "phx_leave".to_string(),
                payload: json!({}),
              };
              self.phoenix_manager.do_send(msg);
            }
            _ => {}
          }
        } else {
          println!("Invalid JSON input: {}", text);
        }
      }
      Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
      Ok(ws::Message::Close(_)) => ctx.stop(),
      _ => {}
    }
  }
}
