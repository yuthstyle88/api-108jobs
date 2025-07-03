use crate::proxy::PhoenixProxy;
use actix::prelude::*;
use log::{error};
use phoenix_channels_client::{Channel, Client, Config, Payload};
use serde_json::json;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;

pub struct PhoenixActor {
  config: Config,
  proxy: PhoenixProxy,
  rx: Option<Receiver<(String, String)>>,
  client: Option<Arc<Mutex<Client>>>,
  _channels: HashMap<String, Arc<Channel>>,
}

impl PhoenixActor {
  pub fn new(config: Config, proxy: PhoenixProxy, rx: Receiver<(String, String)>) -> Self {
    Self {
      config,
      proxy,
      rx: Some(rx),
      client: None,
      _channels: HashMap::new(),
    }
  }
}

impl Actor for PhoenixActor {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    // 1. create and wrap in Arc<Mutex>
    let client = match Client::new(self.config.clone()) {
      Ok(c) => Arc::new(Mutex::new(c)),
      Err(err) => {
        error!("Failed to create Phoenix client: {:?}", err);
        ctx.stop();
        return;
      }
    };

    // 2. save the Arc<Mutex<Client>> in the actor
    self.client = Some(client.clone());

    let proxy = self.proxy.clone();
    let mut rx = match self.rx.take() {
      Some(rx) => rx,
      None => {
        error!("Receiver already taken or missing");
        ctx.stop();
        return;
      }
    };

    let addr = ctx.address();

    // 3. spawn the background task, using the cloned Arc
    actix::spawn(async move {
      // connect
      {
        let mut guard = client.lock().await;
        if let Err(err) = guard.connect().await {
          error!("Failed to connect to Phoenix: {:?}", err);
          addr.do_send(StopActor);
          return;
        }
      } // guard dropped here

      let mut channels = HashMap::<String, Arc<Channel>>::new();

      while let Some((room_id, message)) = rx.recv().await {
        let topic = format!("room:{}", room_id);

        // join or reuse channel
        let channel = match channels.get(&topic) {
          Some(ch) => ch.clone(),
          None => {
            // lock the client for join
            let guard = client.lock().await;
            match guard.join(&topic, Some(Duration::from_secs(10))).await {
              Ok(ch) => {
                let proxy_clone = proxy.clone();
                let topic_clone = topic.clone();

                ch.on("new_message", move |_ch, payload| {
                  let payload = payload.clone();
                  let proxy = proxy_clone.clone();
                  let topic = topic_clone.clone();

                  actix::spawn(async move {
                    let room_id = topic.strip_prefix("room:").unwrap_or(&topic);
                    if let Payload::Value(val) = payload {
                      if let Some(body) = val.get("body").and_then(|v| v.as_str()) {
                        proxy.broadcast_to_room(room_id, body).await;
                      }
                    }
                  });
                })
                    .await
                    .unwrap();

                channels.insert(topic.clone(), ch.clone());
                ch
              }
              Err(e) => {
                error!("Failed to join topic {}: {:?}", topic, e);
                continue;
              }
            }
          }
        };

        // send the message
        if let Err(e) = channel
            .send_noreply("new_message", json!({ "body": message }))
            .await
        {
          error!("Failed to send message to Phoenix: {:?}", e);
        }
      }
    });
  }
}

/// Internal message to stop actor from external spawn
struct StopActor;

impl Message for StopActor {
  type Result = ();
}

impl Handler<StopActor> for PhoenixActor {
  type Result = ();

  fn handle(&mut self, _msg: StopActor, ctx: &mut Self::Context) {
    ctx.stop();
  }
}
