use crate::newtypes::{PersonId, PersonProposalMentionId, ProposalId};
#[cfg(feature = "full")]
use crate::schema::person_proposal_mention;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::proposal::Proposal, foreign_key = comment_id)))]
#[cfg_attr(feature = "full", diesel(table_name = person_proposal_mention))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A person mention.
pub struct PersonProposalMention {
  pub id: PersonProposalMentionId,
  pub recipient_id: PersonId,
  pub comment_id: ProposalId,
  pub read: bool,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_proposal_mention))]
pub struct PersonProposalMentionInsertForm {
  pub recipient_id: PersonId,
  pub comment_id: ProposalId,
  pub read: Option<bool>,
}

#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_proposal_mention))]
pub struct PersonProposalMentionUpdateForm {
  pub read: Option<bool>,
}
