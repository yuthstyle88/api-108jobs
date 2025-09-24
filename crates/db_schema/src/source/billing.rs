use crate::newtypes::{BillingId, ChatRoomId, Coin, CommentId, DbUrl, LocalUserId, PostId};
use chrono::{DateTime, NaiveDate, Utc};
use lemmy_db_schema_file::enums::{BillingStatus, WorkFlowStatus};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::billing;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
#[skip_serializing_none]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = billing))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A billing/invoice created by freelancer for job work.
#[serde(rename_all = "camelCase")]
pub struct Billing {
  pub id: BillingId,
  /// The freelancer who created this billing
  pub freelancer_id: LocalUserId,
  /// The employer who needs to pay this billing
  pub employer_id: LocalUserId,
  /// The job post this billing is for
  pub post_id: PostId,
  /// The comment/proposal this billing relates to
  pub comment_id: Option<CommentId>,
  /// Amount to be paid
  pub amount: Coin,
  /// Description of work to be done
  pub description: String,
  /// Current status of the billing
  #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
  pub status: BillingStatus,
  /// Submitted work description
  pub work_description: Option<String>,
  /// URL to deliverable files
  pub deliverable_url: Option<String>,
  /// When the billing was created
  pub created_at: DateTime<Utc>,
  /// When the billing was last updated
  pub updated_at: Option<DateTime<Utc>>,
  /// When the billing was paid (if paid)
  pub paid_at: Option<DateTime<Utc>>,
  pub room_id: ChatRoomId,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = billing))]
pub struct BillingInsertForm {
  pub freelancer_id: LocalUserId,
  pub employer_id: LocalUserId,
  pub post_id: PostId,
  #[new(default)]
  pub comment_id: Option<CommentId>,
  pub amount: Coin,
  pub description: String,
  #[new(default)]
  pub status: Option<BillingStatus>,
  #[new(default)]
  pub work_description: Option<String>,
  #[new(default)]
  pub deliverable_url: Option<DbUrl>,
  #[new(default)]
  pub created_at: Option<DateTime<Utc>>,
  pub room_id: ChatRoomId,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = billing))]
pub struct BillingUpdateForm {
  pub status: Option<BillingStatus>,
  pub work_description: Option<Option<String>>,
  pub deliverable_url: Option<Option<String>>,
  pub updated_at: Option<DateTime<Utc>>,
  pub paid_at: Option<Option<DateTime<Utc>>>,
}

/// Helper input for creating BillingInsertForm from a quotation context.
/// This stays in the same crate as BillingInsertForm to allow a From impl without orphan rule issues.
#[derive(Clone, Default)]
pub struct BillingFromQuotation {
  pub freelancer_id: LocalUserId,
  pub employer_id: LocalUserId,
  pub post_id: PostId,
  pub comment_id: Option<CommentId>,
  pub amount: Coin,
  pub description: String,
  pub delivery_day: NaiveDate,
  pub room_id: ChatRoomId,
}

impl From<BillingFromQuotation> for BillingInsertForm {
  fn from(v: BillingFromQuotation) -> Self {
    BillingInsertForm {
      freelancer_id: v.freelancer_id,
      employer_id: v.employer_id,
      post_id: v.post_id,
      comment_id: v.comment_id,
      amount: v.amount,
      description: v.description,
      status: Some(BillingStatus::QuotePendingReview),
      work_description: None,
      deliverable_url: None,
      created_at: Some(Utc::now()),
      room_id: v.room_id,
    }
  }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct WorkStep {
  pub seq: i32,
  pub description: String,
  pub amount: Coin, // ใช้ Coin ให้สอดคล้องกับระบบเงินทั้งหมด
  pub working_days: i32,
  #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
  pub status: WorkFlowStatus,
  #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
  pub starting_day: NaiveDate,
  #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
  pub delivery_day: NaiveDate,
}
