use crate::api::{ChatEvent, IncomingEvent, MessageStatus};
use crate::bridge_message::{BridgeMessage, OutboundMessage};
use crate::broker::helper::{get_or_create_channel, send_event_to_channel};
use crate::broker::pending_ack_handler::handle_ack_event;
use crate::broker::phoenix_manager::{PhoenixManager, JOIN_TIMEOUT_SECS};
use crate::broker::presence_manager::{Heartbeat, OnlineJoin};
use crate::impls::AnyIncomingEvent;
use actix::{Context, Handler, ResponseFuture};
use actix::prelude::*;
use actix_broker::{BrokerIssue, SystemBroker};
use chrono::Utc;
use lemmy_db_schema::newtypes::ChatRoomId;
use lemmy_db_schema::source::chat_message::ChatMessageInsertForm;
use lemmy_db_schema::source::chat_participant::ChatParticipant;
use phoenix_channels_client::{ChannelStatus, Event, Payload};
use serde_json;
use std::sync::Arc;
use std::time::Duration;
use serde_json::json;
use tokio::time::timeout;
use lemmy_db_schema::utils::DbPool;
use lemmy_db_views_chat_pending_ack::AckReminderQuery;

// Message to route arbitrary topic emissions back through PhoenixManager (so we can use issue_async safely)
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct EmitTopics {
  pub items: Vec<(String, ChatEvent, serde_json::Value)>,
}

impl Handler<BridgeMessage> for PhoenixManager {
  type Result = ResponseFuture<()>;

  fn handle(&mut self, msg: BridgeMessage, ctx: &mut Context<Self>) -> Self::Result {
    // Convert wire-level IncomingEvent (payload: Value) to strongly-typed AnyIncomingEvent
    let any_event: AnyIncomingEvent = msg.any_event.clone();

    let socket = self.socket.clone();
    let channels = Arc::clone(&self.channels);
    let addr = ctx.address();

    // Helper to issue outbound (to internal broker) using JSON payload
    let issue_outbound =
      |room_id: ChatRoomId, event: ChatEvent, json_payload: serde_json::Value| {
        let out_event = IncomingEvent {
          room_id: room_id.clone(),
          event,
          topic: format!("room:{}", room_id),
          payload: json_payload,
        };
        self.issue_async::<SystemBroker, _>(OutboundMessage { out_event });
      };

    // Note: arbitrary-topic emission is handled via EmitTopics message to avoid borrowing self in async tasks

    match any_event {
      // ---------------------- MESSAGE ----------------------
      AnyIncomingEvent::Message(ev) => {
        if let Some(mut payload) = ev.payload.clone() {
          // mark status as Sent for outbound consumers
          payload.status = Some(MessageStatus::Delivered);
          let json = serde_json::to_value(&payload).unwrap_or(serde_json::Value::Null);
          issue_outbound(ev.room_id.clone(), ChatEvent::Message, json);

          // 2) persist if has id
          if payload.id.is_some() {
            let mut this = self.clone();
            let insert_data = ChatMessageInsertForm {
              msg_ref_id: payload.id.clone(),
              room_id: ev.room_id.clone(),
              sender_id: payload.sender_id,
              content: payload.content.clone(),
              status: 1,
              created_at: payload.created_at.clone(),
              updated_at: None,
            };
            actix::spawn(async move {
              if let Err(e) = this.add_messages_to_room(Some(insert_data)).await {
                tracing::error!("Failed to store message in Redis: {}", e);
              }
            });

            // 1) Issue messageAck to client
            let client_id_json = serde_json::to_value(&payload.id).unwrap_or(serde_json::Value::Null);
            issue_outbound(ev.room_id.clone(), ChatEvent::MessageAck, serde_json::json!({
              "clientId": client_id_json
            }));

            // 2) Enqueue pending ack (idempotent)
            let pool_owned = self.pool.clone();
            let room_id_for_enqueue = ev.room_id.clone();
            let sender_id_for_enqueue = payload.sender_id;
            // Parse client_id as UUID string if necessary
            let client_id_for_enqueue_opt = payload.id.as_ref().and_then(|v| Some(v.as_str())).and_then(|s| uuid::Uuid::parse_str(s).ok());

            if let Some(client_id_for_enqueue) = client_id_for_enqueue_opt {
              actix::spawn(async move {
                let mut pool = DbPool::Pool(&pool_owned);
                let _ = lemmy_db_views_chat_pending_ack::enqueue_pending(
                  &mut pool,
                  room_id_for_enqueue,
                  sender_id_for_enqueue,
                  Some(client_id_for_enqueue),
                ).await;
              });
            }

            // 3) Emit lightweight user-level chats signal to each participant (except sender)
            let pool_owned = self.pool.clone();
            let room_id_signal = ev.room_id.clone();
            let sender_id_signal = payload.sender_id;
            let last_message_id = payload.id.clone();
            let last_message_at = payload.created_at.clone();
            actix::spawn(async move {
              let mut db = DbPool::Pool(&pool_owned);
              // Fetch participants for the room
              if let Ok(participants) = ChatParticipant::list_participants_for_rooms(&mut db, &[room_id_signal.clone()]).await {
                for p in participants.into_iter().filter(|p| sender_id_signal.map(|sid| sid != p.member_id).unwrap_or(true)) {
                  // TODO: compute real unreadCount; default to 0 for now
                  let payload = json!({
                    "version": 1,
                    "roomId": room_id_signal,
                    "lastMessageId": last_message_id,
                    "lastMessageAt": last_message_at,
                    "unreadCount": 0,
                    "senderId": sender_id_signal,
                  });
                  let topic = format!("user:{}:events", p.member_id.0);
                  let _ = addr.do_send(EmitTopics { items: vec![(topic, ChatEvent::ChatsSignal, payload)] });
                }
              }
            });
          }
        } else {
          // still notify listeners to keep flow consistent
          issue_outbound(ev.room_id, ChatEvent::Message, serde_json::Value::Null);
        }
        Box::pin(async move {})
      }

      // ---------------------- TYPING ----------------------
      AnyIncomingEvent::Typing(ev) => {
        let json = if let Some(m) = ev.payload.clone() {
          serde_json::to_value(&m).unwrap_or(serde_json::Value::Null)
        } else {
          json!({"typing": true})
        };
        issue_outbound(ev.room_id, ChatEvent::Typing, json);
        Box::pin(async move {})
      }

      // ---------------------- UPDATE ----------------------
      AnyIncomingEvent::Update(ev) => {
        let json = ev
          .payload
          .as_ref()
          .map(|m| serde_json::to_value(m).unwrap_or(serde_json::Value::Null))
          .unwrap_or(serde_json::Value::Null);
        issue_outbound(ev.room_id, ChatEvent::Update, json);
        Box::pin(async move {})
      }

      // ---------------------- READ ----------------------
      AnyIncomingEvent::Read(ev) => {
        let mut json = ev
          .payload
          .as_ref()
          .map(|p| serde_json::to_value(p).unwrap_or(serde_json::Value::Null))
          .unwrap_or(serde_json::Value::Null);

        let mut this = self.clone();
        let room_id_clone = ev.room_id.clone();
        let payload_clone = ev.payload.clone();
        let read_at = Utc::now();

        actix::spawn(async move {
          if let Err(e) = this.handle_read_event(room_id_clone, payload_clone, Some(read_at)) {
            tracing::error!("Failed to handle read up to event: {}", e);
          }
        });
        // Add timestamp to JSON response
        if let Some(obj) = json.as_object_mut() {
          obj.insert(
            "updatedAt".to_string(),
            serde_json::Value::String(read_at.to_rfc3339()),
          );
        }

        issue_outbound(ev.room_id, ChatEvent::ReadUpTo, json); // now safe to use original

        // Emit reset signal for the reader on user topic (unreadCount = 0)
        if let Some(p) = ev.payload.clone() {
          let topic = format!("user:{}:events", p.reader_id.0);
          let payload = json!({
            "version": 1,
            "roomId": p.room_id,
            "unreadCount": 0,
          });
          let _ = addr.do_send(EmitTopics { items: vec![(topic, ChatEvent::ChatsSignal, payload)] });
        }
        Box::pin(async move {})
      }
      AnyIncomingEvent::AckConfirm(ev) => {
        // 2) NEW: run DB-side removal from pending_sender_ack (idempotent)
        //    Use helper to parse camelCase payload and call chat_pending_ack::ack_confirm.
        //    Fire-and-forget to avoid blocking the actor loop.
        {
          // Clone the pool to avoid borrowing `self` across the spawned task.
          let pool_owned = self.pool.clone();
          let any = AnyIncomingEvent::AckConfirm(ev.clone());
          actix::spawn(async move {
            let mut pool = DbPool::Pool(&pool_owned);
            let _ = handle_ack_event(any, &mut pool).await;
          });
        }
        Box::pin(async move {})
      }
      AnyIncomingEvent::SyncPending(ev) => {
        if let Some(p) = ev.payload.clone() {
          let pool_owned = self.pool.clone();
          let topic = ev.topic.clone();
          actix::spawn(async move {
            let mut pool = DbPool::Pool(&pool_owned);
            match lemmy_db_views_chat_pending_ack::ack_reminder(
              &mut pool,
              &AckReminderQuery { room_id: p.room_id, sender_id: p.sender_id, limit: Some(200) }
            ).await {
              Ok(reminder) => {
                let _ = addr.do_send(EmitTopics {
                  items: vec![(
                    topic,
                    ChatEvent::SyncPending,
                    json!({ "clientIds": reminder.client_ids })
                  )]
                });
              }
              Err(e) => {
                tracing::error!("ack_reminder failed: {}", e);
              }
            }
          });
        }
        Box::pin(async move {})
      }

      // ---------------------- JOIN / HEARTBEAT / ACTIVE_ROOMS / LEAVE ----------------------
      AnyIncomingEvent::Join(ev) => {
        if let Some(payload) = ev.payload.clone() {
          let channel_name = ev.topic.clone();
          let outbound_event = ev.event.clone();
          let room_id = ev.room_id.clone();
          let sender_id = payload.sender_id;
          // keep original payload as JSON once
          let content_json = serde_json::to_value(&payload).unwrap_or(serde_json::Value::Null);
          let this = self.clone();

          Box::pin(async move {
            if let Ok(arc_chan) = get_or_create_channel(channels, socket, &channel_name).await {
              // Build once
              let phoenix_event = Event::from_string(outbound_event.to_string_value());
              let payload_bytes = serde_json::to_vec(&content_json).unwrap_or_default();
              let payload: Payload = Payload::binary_from_bytes(payload_bytes);

              // Try to read current status quickly (non-blocking semantics)
              let status: Option<ChannelStatus> =
                timeout(Duration::from_millis(300), arc_chan.statuses().status())
                  .await
                  .ok()
                  .and_then(|res| res.ok());

              // Mark presence (always)
              let _ = this
                .presence
                .send(OnlineJoin {
                  room_id,
                  local_user_id: sender_id,
                  started_at: Utc::now(),
                })
                .await;

              // Join only if not already joined (or unknown status)
              if !matches!(status, Some(ChannelStatus::Joined)) {
                let _ = arc_chan.join(Duration::from_secs(JOIN_TIMEOUT_SECS)).await;
              }

              // Cast the event
              send_event_to_channel(arc_chan, phoenix_event, payload).await;
            }
          })
        } else {
          Box::pin(async move {})
        }
      }
      AnyIncomingEvent::Heartbeat(ev) => {
        if let Some(payload) = ev.payload.clone() {
          let sender_id = payload.sender_id;
          let this = self.clone();
          Box::pin(async move {
            tracing::debug!("Heartbeat");
            let _ = this
              .presence
              .send(Heartbeat {
                local_user_id: sender_id,
                client_time: None,
              })
              .await;
          })
        } else {
          Box::pin(async move {})
        }
      }
      AnyIncomingEvent::ActiveRooms(_ev) => Box::pin(async move {
        tracing::debug!("ActiveRooms");
      }),
      AnyIncomingEvent::Leave(_ev) => Box::pin(async move {
        tracing::debug!("Leave");
      }),

      AnyIncomingEvent::Unknown => Box::pin(async move {}),
      _ => Box::pin(async move {})
    }
  }
}

impl Handler<EmitTopics> for PhoenixManager {
  type Result = ResponseFuture<()>;

  fn handle(&mut self, msg: EmitTopics, _ctx: &mut Context<Self>) -> Self::Result {
    for (topic, event, payload) in msg.items.into_iter() {
      let room_id: ChatRoomId =
        ChatRoomId::from_channel_name(topic.as_str()).unwrap_or(ChatRoomId(topic.clone()));
      let out_event = IncomingEvent {
        room_id,
        event: event.clone(),
        topic: topic.clone(),
        payload,
      };
      self.issue_async::<SystemBroker, _>(OutboundMessage { out_event });
    }
    Box::pin(async move {})
  }
}
