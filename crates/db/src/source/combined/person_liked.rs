use crate::newtypes::{PersonId, PersonLikedCombinedId, PostId, ProposalId};
#[cfg(feature = "full")]
use crate::schema::person_liked_combined;
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
#[cfg_attr(feature = "full", diesel(table_name = person_liked_combined))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = person_liked_combined_keys))]
/// A combined person_liked table.
pub struct PersonLikedCombined {
  pub id: PersonLikedCombinedId,
  pub liked_at: DateTime<Utc>,
  pub like_score: i16,
  pub person_id: PersonId,
  pub post_id: Option<PostId>,
  pub proposal_id: Option<ProposalId>,
}
