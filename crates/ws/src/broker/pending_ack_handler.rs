//! Chat Pending ACK event handler
//!
//! This module exposes a single entry point `handle_ack_event(...)` which returns
//! a `FastJobResult<()>` for the broker/actor to await or spawn.

use crate::impls::AnyIncomingEvent;
use lemmy_db_schema::utils::DbPool; // use DbPool to match db_views function signature
use lemmy_db_views_chat_pending_ack::{AckConfirmRequest, AckConfirmResponse};
use lemmy_utils::error::FastJobResult;
use uuid::Uuid;

/// Handle AckConfirm/SyncPending/Unknown events from the client.
/// - Never panics; errors are swallowed (best-effort) to avoid breaking the WS stream.
pub async fn handle_ack_event(any_event: AnyIncomingEvent, pool: &mut DbPool<'_>) -> FastJobResult<()> {
    match any_event {
        AnyIncomingEvent::AckConfirm(ev) => {
            // Payload is already deserialized by upper layer. If missing, no-op.
            if let Some(p) = ev.payload {
                let room_id = p.room_id;       // ChatRoomId
                let sender_id = p.sender_id;   // LocalUserId

                // NOTE: Current payload type for `client_ids` in this crate is not UUID.
                // Until the FE/DTO aligns to Vec<Uuid>, we submit an empty list (idempotent call).
                // This will be a no-op delete and keeps the flow safe.
                let client_ids: Vec<Uuid> = Vec::new();

                let req = AckConfirmRequest { room_id, sender_id, client_ids };
                // Best-effort; idempotent. Swallow error to avoid breaking the WS stream.
                let _res: Result<AckConfirmResponse, _> = lemmy_db_views_chat_pending_ack::ack_confirm(pool, &req).await;
            }
            Ok(())
        }

        AnyIncomingEvent::SyncPending(_ev) => {
            // TODO(next): call ack_reminder + reconciliation (nack:missing) here.
            Ok(())
        }

        AnyIncomingEvent::Unknown => Ok(()),
        _ =>  Ok(())
    }
}