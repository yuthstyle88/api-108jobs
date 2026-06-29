use crate::newtypes::{LanguageId, PersonId, PostId, ProposalId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  crate::newtypes::LtreeDef,
  crate::schema::{proposal, proposal_actions},
  diesel_ltree::Ltree,
  i_love_jesus::CursorKeysModule,
};
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", diesel(table_name = proposal))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = proposal_keys))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A proposal.
#[serde(rename_all = "camelCase")]
pub struct Proposal {
  pub id: ProposalId,
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub content: String,
  /// Whether the proposal has been removed.
  pub removed: bool,
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  /// Whether the proposal has been deleted by its creator.
  pub deleted: bool,
  #[cfg(feature = "full")]
  #[cfg_attr(feature = "full", serde(with = "LtreeDef"))]
  #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
  /// The path / tree location of a proposal, separated by dots, ending with the proposal's id. Ex:
  /// 0.24.27
  pub path: Ltree,
  #[cfg(not(feature = "full"))]
  pub path: String,
  /// Whether the proposal has been distinguished(speaking officially) by a mod.
  pub distinguished: bool,
  pub language_id: LanguageId,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  /// The total number of children in this proposal branch.
  pub child_count: i32,
  #[serde(skip)]
  pub hot_rank: f64,
  #[serde(skip)]
  pub controversy_rank: f64,
  pub report_count: i16,
  pub unresolved_report_count: i16,
  pub pending: bool,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(
  feature = "full",
  derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = proposal))]
#[cfg_attr(feature = "full", serde(rename_all = "camelCase"))]
pub struct ProposalInsertForm {
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub content: String,
  #[new(default)]
  pub removed: Option<bool>,
  #[new(default)]
  pub published_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub updated_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub deleted: Option<bool>,
  #[new(default)]
  pub distinguished: Option<bool>,
  #[new(default)]
  pub language_id: Option<LanguageId>,
  #[new(default)]
  pub pending: Option<bool>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset, Serialize, Deserialize))]
#[cfg_attr(feature = "full", diesel(table_name = proposal))]
#[cfg_attr(feature = "full", serde(rename_all = "camelCase"))]
pub struct ProposalUpdateForm {
  pub content: Option<String>,
  pub removed: Option<bool>,
  // Don't use a default Utc::now here, because the create function does a lot of proposal updates
  pub updated_at: Option<Option<DateTime<Utc>>>,
  pub deleted: Option<bool>,
  pub distinguished: Option<bool>,
  pub language_id: Option<LanguageId>,
  pub pending: Option<bool>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::proposal::Proposal, foreign_key = proposal_id)))]
#[cfg_attr(feature = "full", diesel(table_name = proposal_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, proposal_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = proposal_actions_keys))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ProposalActions {
  #[serde(skip)]
  pub person_id: PersonId,
  #[serde(skip)]
  pub proposal_id: ProposalId,
  /// The like / score for the proposal.
  pub like_score: Option<i16>,
  /// When the proposal was liked.
  pub liked_at: Option<DateTime<Utc>>,
  /// When the proposal was saved.
  pub saved_at: Option<DateTime<Utc>>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(
  feature = "full",
  derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = proposal_actions))]
pub struct ProposalLikeForm {
  pub person_id: PersonId,
  pub proposal_id: ProposalId,
  pub like_score: i16,
  #[new(value = "Utc::now()")]
  pub liked_at: DateTime<Utc>,
}

#[derive(derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = proposal_actions))]
pub struct ProposalSavedForm {
  pub person_id: PersonId,
  pub proposal_id: ProposalId,
  #[new(value = "Utc::now()")]
  pub saved_at: DateTime<Utc>,
}
