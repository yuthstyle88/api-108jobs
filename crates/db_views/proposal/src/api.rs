use crate::{ProposalSlimView, ProposalView};
use app_108jobs_db::{
  enums::{ListingType, ProposalSortType},
  newtypes::{CategoryId, LanguageId, PaginationCursor, PostId, ProposalId},
};
use app_108jobs_db_views_vote::VoteView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// A proposal response.
pub struct ProposalResponse {
  pub proposal_view: ProposalView,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Create a proposal.
pub struct CreateComment {
  pub content: String,
  pub post_id: PostId,
  pub parent_id: Option<ProposalId>,
  pub language_id: Option<LanguageId>,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Create a proposal.
pub struct CreateCommentRequest {
  pub content: String,
  pub post_id: PostId,
  pub parent_id: Option<ProposalId>,
  pub language_id: LanguageId,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Like a proposal.
pub struct CreateCommentLike {
  pub comment_id: ProposalId,
  /// Must be -1, 0, or 1 .
  pub score: i16,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Like a proposal.
pub struct CreateCommentLikeRequest {
  pub comment_id: ProposalId,
  /// Must be -1, 0, or 1 .
  pub score: i16,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Delete your own proposal.
pub struct DeleteComment {
  pub comment_id: ProposalId,
  pub deleted: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Delete your own proposal.
pub struct DeleteCommentRequest {
  pub comment_id: ProposalId,
  pub deleted: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Distinguish a proposal (IE speak as moderator).
pub struct DistinguishComment {
  pub comment_id: ProposalId,
  pub distinguished: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Fetch an individual proposal.
pub struct GetComment {
  pub id: ProposalId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Get a list of comments.
pub struct GetComments {
  pub type_: Option<ListingType>,
  pub sort: Option<ProposalSortType>,
  /// Filter to within a given time range, in seconds.
  /// IE 60 would give results for the past minute.
  pub time_range_seconds: Option<i32>,
  pub max_depth: Option<i32>,
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
  pub category_id: Option<CategoryId>,
  pub category_name: Option<String>,
  pub post_id: Option<PostId>,
  pub parent_id: Option<ProposalId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// The proposal list response.
pub struct GetCommentsResponse {
  pub proposals: Vec<ProposalView>,
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// A slimmer proposal list response, without the post or category.
pub struct GetCommentsSlimResponse {
  pub proposals: Vec<ProposalSlimView>,
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// List proposal likes. Admins-only.
pub struct ListCommentLikes {
  pub comment_id: ProposalId,
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// The proposal likes response
pub struct ListCommentLikesResponse {
  pub comment_likes: Vec<VoteView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Purges a proposal from the database. This will delete all content attached to that proposal.
pub struct PurgeComment {
  pub comment_id: ProposalId,
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Remove a proposal (only doable by mods).
pub struct RemoveComment {
  pub comment_id: ProposalId,
  pub removed: bool,
  pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Save / bookmark a proposal.
pub struct SaveComment {
  pub comment_id: ProposalId,
  pub save: bool,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Edit a proposal.
pub struct EditComment {
  pub comment_id: ProposalId,
  pub content: Option<String>,
  pub language_id: Option<LanguageId>,
  pub budget: i32,
  pub working_days: i32,
  pub brief_url: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Edit a proposal.
pub struct EditCommentRequest {
  pub comment_id: ProposalId,
  pub content: Option<String>,
  pub language_id: Option<LanguageId>,
  pub budget: i32,
  pub working_days: i32,
  pub brief_url: String,
}
