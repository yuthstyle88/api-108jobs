//! Chat Pending ACK db-views
//!
//! This crate defines:
//! - `api` (DTOs / request & response types) — always available
//! - `impls` (DB-facing functions) — compiled only with the `full` feature

/// Public API types (DTOs)
pub mod api;

/// Database-backed implementations (only when `full` is enabled)
#[cfg(feature = "full")]
pub mod impls;

// Re-export API DTOs for convenient use: `chat_pending_ack::ListChatPendingAckQuery`, etc.
pub use api::{
    AckConfirmRequest,
    AckConfirmResponse,
    AckReminderQuery,
    AckReminderResponse,
    ChatPendingAckItem,
    LastReadChatPendingAckQuery,
    ListChatPendingAckQuery,
    ListChatPendingAckResponse,
};

// Re-export DB functions when available
#[cfg(feature = "full")]
pub use impls::{
    ack_confirm,
    ack_reminder,
    list_pending_ack,
    enqueue_pending,
};
