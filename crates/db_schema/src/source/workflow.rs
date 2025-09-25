use crate::newtypes::{BillingId, ChatRoomId, PostId, WorkflowId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::WorkFlowStatus;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::workflow;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = workflow))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct Workflow {
  pub id: WorkflowId,
  pub post_id: PostId,
  pub seq_number: i16,
  pub status: WorkFlowStatus,
  pub revision_required: bool,
  pub revision_count: i16,
  pub revision_reason: Option<String>,
  pub deliverable_version: i16,
  pub deliverable_submitted_at: Option<DateTime<Utc>>,
  pub deliverable_accepted: bool,
  pub accepted_at: Option<DateTime<Utc>>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  pub room_id: ChatRoomId,
  pub deliverable_url: Option<String>,
  pub active: bool,
  pub has_proposed_quote: bool,
  pub status_before_cancel: Option<WorkFlowStatus>,
  pub billing_id: Option<BillingId>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = workflow))]
pub struct WorkflowInsertForm {
  pub post_id: PostId,
  pub seq_number: i16,
  #[new(default)]
  pub status: Option<WorkFlowStatus>,
  #[new(default)]
  pub revision_required: Option<bool>,
  #[new(default)]
  pub revision_count: Option<i16>,
  #[new(default)]
  pub revision_reason: Option<Option<String>>,
  #[new(default)]
  pub deliverable_version: Option<i16>,
  #[new(default)]
  pub deliverable_submitted_at: Option<Option<DateTime<Utc>>>,
  #[new(default)]
  pub deliverable_accepted: Option<bool>,
  #[new(default)]
  pub accepted_at: Option<Option<DateTime<Utc>>>,
  #[new(default)]
  pub created_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub updated_at: Option<Option<DateTime<Utc>>>,
  pub room_id: ChatRoomId,
  #[new(default)]
  pub deliverable_url: Option<Option<String>>,
  #[new(default)]
  pub active: Option<bool>,
  #[new(default)]
  pub has_proposed_quote: Option<bool>,
  #[new(default)]
  pub status_before_cancel: Option<Option<WorkFlowStatus>>,
  #[new(default)]
  pub billing_id: Option<Option<BillingId>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = workflow))]
pub struct WorkflowUpdateForm {
  pub status: Option<WorkFlowStatus>,
  pub revision_required: Option<bool>,
  pub revision_count: Option<i16>,
  pub revision_reason: Option<Option<String>>,
  pub deliverable_version: Option<i16>,
  pub deliverable_submitted_at: Option<Option<DateTime<Utc>>>,
  pub deliverable_accepted: Option<bool>,
  pub accepted_at: Option<Option<DateTime<Utc>>>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
  pub room_id: Option<ChatRoomId>,
  pub deliverable_url: Option<Option<String>>,
  pub active: Option<bool>,
  pub has_proposed_quote: Option<bool>,
  pub status_before_cancel: Option<Option<WorkFlowStatus>>,
  pub billing_id: Option<Option<BillingId>>,
}


