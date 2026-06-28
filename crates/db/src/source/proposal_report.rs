use crate::newtypes::{PersonId, ProposalId, ProposalReportId};
#[cfg(feature = "full")]
use crate::schema::proposal_report;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::proposal::Proposal, foreign_key = comment_id)))]
#[cfg_attr(feature = "full", diesel(table_name = proposal_report))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A proposal report.
#[serde(rename_all = "camelCase")]
pub struct ProposalReport {
  pub id: ProposalReportId,
  pub creator_id: PersonId,
  pub comment_id: ProposalId,
  pub original_comment_text: String,
  pub reason: String,
  pub resolved: bool,
  pub resolver_id: Option<PersonId>,
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  pub violates_instance_rules: bool,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = proposal_report))]
pub struct ProposalReportForm {
  pub creator_id: PersonId,
  pub comment_id: ProposalId,
  pub original_comment_text: String,
  pub reason: String,
  pub violates_instance_rules: bool,
}
