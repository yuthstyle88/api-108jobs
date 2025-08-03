use crate::newtypes::{BillingId, LocalUserId, CommentId, PostId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::BillingStatus;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::billing;

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
  pub amount: f64,
  /// Description of work to be done
  pub description: String,
  /// Maximum number of revisions allowed
  pub max_revisions: i32,
  /// Current number of revisions used
  pub revisions_used: i32,
  /// Current status of the billing
  pub status: BillingStatus,
  /// Submitted work description
  pub work_description: Option<String>,
  /// URL to deliverable files
  pub deliverable_url: Option<String>,
  /// Latest revision feedback from employer
  pub revision_feedback: Option<String>,
  /// When the billing was created
  pub created_at: DateTime<Utc>,
  /// When the billing was last updated
  pub updated_at: Option<DateTime<Utc>>,
  /// When the billing was paid (if paid)
  pub paid_at: Option<DateTime<Utc>>,
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
  pub amount: f64,
  pub description: String,
  pub max_revisions: i32,
  #[new(default)]
  pub revisions_used: Option<i32>,
  #[new(default)]
  pub status: Option<BillingStatus>,
  #[new(default)]
  pub created_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = billing))]
pub struct BillingUpdateForm {
  pub status: Option<BillingStatus>,
  pub revisions_used: Option<i32>,
  pub work_description: Option<Option<String>>,
  pub deliverable_url: Option<Option<String>>,
  pub revision_feedback: Option<Option<String>>,
  pub updated_at: Option<DateTime<Utc>>,
  pub paid_at: Option<Option<DateTime<Utc>>>,
}