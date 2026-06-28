use crate::newtypes::{PersonId, ProposalId, ProposalReplyId};
#[cfg(feature = "full")]
use crate::schema::proposal_reply;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::proposal::Proposal, foreign_key = comment_id)))]
#[cfg_attr(feature = "full", diesel(table_name = proposal_reply))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A proposal reply.
#[serde(rename_all = "camelCase")]
pub struct ProposalReply {
  pub id: ProposalReplyId,
  pub recipient_id: PersonId,
  pub comment_id: ProposalId,
  pub read: bool,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = proposal_reply))]
pub struct ProposalReplyInsertForm {
  pub recipient_id: PersonId,
  pub comment_id: ProposalId,
  pub read: Option<bool>,
}

#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = proposal_reply))]
pub struct ProposalReplyUpdateForm {
  pub read: Option<bool>,
}
