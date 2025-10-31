use chrono::{DateTime, Utc};
use lemmy_db_schema::newtypes::{
  ChatRoomId, LocalUserId, PaginationCursor,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
/// GET /api/chat/pending-ack
/// Query pending ACK tokens/items for a given stream (roomId + senderId).
/// - Typical use: server → client on join (ackReminder), or admin/debug UI
pub struct ListChatPendingAckQuery {
  /// Chat room to inspect
  pub room_id: ChatRoomId,
  /// Sender (local user) whose pending ACKs we list
  pub sender_id: LocalUserId,
  /// Optional max number of items (default e.g. 100)
  pub limit: Option<i64>,
  /// Return only items created before this cursor/timestamp or server id (optional)
  pub before: Option<PaginationCursor>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// A single pending ACK item (for display/inspection)
pub struct ChatPendingAckItem {
  pub room_id: ChatRoomId,
  pub sender_id: LocalUserId,
  pub client_id: Uuid,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// Response for listing pending ACK items
pub struct ListChatPendingAckResponse {
  pub items: Vec<ChatPendingAckItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
/// POST /api/chat/pending-ack/confirm
/// Confirm (ack-of-ack) a batch of pending ACKs by clientId.
/// This is idempotent: resending the same ids is safe.
pub struct AckConfirmRequest {
  pub room_id: ChatRoomId,
  pub sender_id: LocalUserId,
  pub client_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// Result of confirming pending ACKs
pub struct AckConfirmResponse {
  /// Number of rows removed from the pending queue
  pub removed: usize,
  /// Client IDs that were not found in the pending queue (for transparency)
  pub not_found: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
/// GET /api/chat/pending-ack/reminder
/// Ask server which ACKs are still pending for this stream (used on join/reconnect).
pub struct AckReminderQuery {
  pub room_id: ChatRoomId,
  pub sender_id: LocalUserId,
  /// Optional cap for how many tokens/ids to return
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// Response for ack-reminder: which clientIds need ackConfirm
pub struct AckReminderResponse {
  pub client_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastReadChatPendingAckQuery {
  pub room_id: ChatRoomId,
  pub sender_id: LocalUserId,
}

// ---------------------------------------------------------------------------
// REST Endpoint Specs (to be implemented in the API server crate)
//
// GET  /api/chat/pending-ack
//   - query: ListChatPendingAckQuery
//   - resp : ListChatPendingAckResponse
//
// GET  /api/chat/pending-ack/reminder
//   - query: AckReminderQuery
//   - resp : AckReminderResponse
//
// POST /api/chat/pending-ack/confirm
//   - body : AckConfirmRequest
//   - resp : AckConfirmResponse
//
// GET  /api/chat/pending-ack/last-read
//   - query: LastReadChatPendingAckQuery
//   - resp : { lastSseq: i64 }  // define in the API server crate if needed
//
// Notes:
// - All operations are idempotent.
// - Server is authoritative; client must send ackConfirm after messageAck.
// - Pagination is optional; default limit should be sane (e.g., 100).
// ---------------------------------------------------------------------------