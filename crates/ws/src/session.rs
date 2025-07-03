use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use actix::prelude::*;
use actix_broker::BrokerIssue;
use actix_web::web::Data;
use actix_web_actors::ws;
use diesel_async::AsyncPgConnection;
use phoenix_channels_client::{Client, Config};
use serde_json::json;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::utils::{get_conn, ActualDbPool, DbConn, DbPool};
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use crate::{
    message::{ChatMessage, JoinRoom, LeaveRoom, ListRooms, SendMessage},
    server::WsChatServer,
};

pub struct WsChatSession {
    id: u64,
    room: String,
    name: Option<String>,
    context: Data<FastJobContext>,
}

impl WsChatSession {
    pub fn new(context: Data<FastJobContext>) -> Self {
        Self {
            id: 0,
            room: "main".to_string(),
            name: None,
            context,
        }
    }
    pub fn join_room(&mut self, room_name: &str, ctx: &mut ws::WebsocketContext<Self>) {
        {
            let mut config = Config::new("ws://127.0.0.1:4000/socket/websocket").unwrap();
            config.set("shared_secret", "supersecret");

            // Create a client
            let mut client = Client::new(config).unwrap();
            let topic = format!("room:{}", room_name);
            // Connect the client
            tokio::spawn(async move {
                client.connect().await.unwrap();
                let channel = client.join(&topic, Some(Duration::from_secs(15))).await.unwrap();
            });
        }
        let room_name = room_name.to_owned();

        // First send a leave message for the current room
        let leave_msg = LeaveRoom(self.room.clone(), self.id);

        // issue_sync comes from having the `BrokerIssue` trait in scope.
        self.issue_system_sync(leave_msg, ctx);

        // Then send a join message for the new room
        let join_msg = JoinRoom(
            room_name.to_owned(),
            self.name.clone(),
            ctx.address().recipient(),
        );

        WsChatServer::from_registry()
            .send(join_msg)
            .into_actor(self)
            .then(|id, act, _ctx| {
                if let Ok(id) = id {
                    act.id = id;
                    act.room = room_name;
                }

                fut::ready(())
            })
            .wait(ctx);
    }

        pub fn list_rooms(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
            WsChatServer::from_registry()
                .send(ListRooms)
                .into_actor(self)
                .then(|res, _, ctx| {
                    if let Ok(rooms) = res {
                        for room in rooms {
                            ctx.text(room);
                        }
                    }

                    fut::ready(())
                })
                .wait(ctx);
        }

        pub fn send_msg(&self, msg: &str) {
            {
                let mut config = Config::new("ws://127.0.0.1:4000/socket/websocket").unwrap();
                config.set("shared_secret", "supersecret");

                // Create a client
                let mut client = Client::new(config).unwrap();
                let topic = format!("room:{}", "b8d9a8a5-c296-4d31-8998-e3a76b6eafa1");
                // Connect the client
                tokio::spawn(async move {
                    client.connect().await.unwrap();
                    let channel = client.join(&topic, Some(Duration::from_secs(15))).await.unwrap();
                    let result = channel.send("send_reply", json!({ "name": "foo", "message": "hi"})).await.unwrap();
                });
            }
            let content = format!(
                "{}: {msg}",
                self.name.clone().unwrap_or_else(|| "anon".to_owned()),
            );

            let msg = SendMessage(self.room.clone(), self.id, content);

            // issue_async comes from having the `BrokerIssue` trait in scope.
            self.issue_system_async(msg);
        }
    }

    impl Actor for WsChatSession {
        type Context = ws::WebsocketContext<Self>;

        fn started(&mut self, ctx: &mut Self::Context) {
            self.join_room("b8d9a8a5-c296-4d31-8998-e3a76b6eafa1", ctx);
        }

        fn stopped(&mut self, _ctx: &mut Self::Context) {
            log::info!(
            "WsChatSession closed for {}({}) in room {}",
            self.name.clone().unwrap_or_else(|| "anon".to_owned()),
            self.id,
            self.room
        );
        }
    }

    impl Handler<ChatMessage> for WsChatSession {
        type Result = ();

        fn handle(&mut self, msg: ChatMessage, ctx: &mut Self::Context) {
            ctx.text(msg.0);
        }
    }

    impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
        fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
            let msg = match msg {
                Err(_) => {
                    ctx.stop();
                    return;
                }
                Ok(msg) => msg,
            };

            match msg {
                ws::Message::Text(text) => {
                    let msg = text.trim();

                    if msg.starts_with('/') {
                        let mut command = msg.splitn(2, ' ');

                        match command.next() {
                            Some("/list") => self.list_rooms(ctx),

                            Some("/join") => {
                                if let Some(room_name) = command.next() {
                                    self.join_room(room_name, ctx);
                                } else {
                                    ctx.text("!!! room name is required");
                                }
                            }

                            Some("/name") => {
                                if let Some(name) = command.next() {
                                    self.name = Some(name.to_owned());
                                    ctx.text(format!("name changed to: {name}"));
                                } else {
                                    ctx.text("!!! name is required");
                                }
                            }

                            _ => ctx.text(format!("!!! unknown command: {msg:?}")),
                        }

                        return;
                    }
                    self.send_msg(msg);
                }
                ws::Message::Close(reason) => {
                    ctx.close(reason);
                    ctx.stop();
                }
                _ => {}
            }
        }
    }