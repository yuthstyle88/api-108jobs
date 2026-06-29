use crate::newtypes::{
  InboxCombinedId, PersonPostMentionId, PersonProposalMentionId, ProposalReplyId,
};
#[cfg(feature = "full")]
use crate::schema::inbox_combined;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = inbox_combined))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = inbox_combined_keys))]
/// A combined inbox table.
pub struct InboxCombined {
  pub id: InboxCombinedId,
  pub published_at: DateTime<Utc>,
  pub proposal_reply_id: Option<ProposalReplyId>,
  pub person_proposal_mention_id: Option<PersonProposalMentionId>,
  pub person_post_mention_id: Option<PersonPostMentionId>,
}
