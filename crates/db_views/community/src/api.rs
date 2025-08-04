use crate::{CommunityNodeView, CommunityView};
use lemmy_db_schema::{
  newtypes::{CommunityId, LanguageId, PaginationCursor, TagId},
  source::site::Site,
  CommunitySortType,
};
use lemmy_db_schema_file::enums::{CommunityVisibility, ListingType};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Parameter for setting community icon or banner. Can't use POST data here as it already contains
/// the image data.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct CommunityIdQuery {
  pub id: CommunityId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A simple community response.
#[serde(rename_all = "camelCase")]
pub struct CommunityResponse {
  pub community_view: CommunityView,
  pub discussion_languages: Vec<LanguageId>,
}

#[skip_serializing_none]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
/// Create a community.
#[serde(rename_all = "camelCase")]
pub struct CreateCommunity {
  /// The unique name.
  pub name: String,
  /// A longer title (can be seen as subname)
  pub title: String,
  /// A sidebar for the community in markdown.
  pub sidebar: Option<String>,
  /// A shorter, one line description of your community.
  pub description: Option<String>,
  /// An icon URL.
  pub icon: Option<String>,
  /// A banner URL.
  pub banner: Option<String>,
  /// Whether its self-promotion community.
  pub self_promotion: Option<bool>,
  /// Whether to restrict posting only to moderators.
  pub posting_restricted_to_mods: Option<bool>,
  pub discussion_languages: Option<Vec<LanguageId>>,
  pub visibility: Option<CommunityVisibility>,
  pub is_new: Option<bool>,
  pub parent_id: Option<CommunityId>,
}

#[skip_serializing_none]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
/// Create a community (this is the replacement of a category and subcategory of a job).
pub struct CreateCommunityRequest {
  /// The unique name.
  pub name: Option<String>,
  /// A longer title (can be seen as subname)
  pub title: Option<String>,
  /// A shorter, one-line description of your category or subcategory.
  pub description: Option<String>,
  /// An icon URL.
  pub icon: Option<String>,
  /// A banner URL.
  pub banner: Option<String>,
  /// Whether its an self promotion category.
  pub self_promotion: Option<bool>,
  /// Whether its new or not.
  pub is_new: Option<bool>,
  /// If parent_id is None -> Category else -> Subcategory
  pub parent_id: Option<CommunityId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Create a tag for a community.
pub struct CreateCommunityTag {
  pub community_id: CommunityId,
  pub display_name: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Delete your own community.
pub struct DeleteCommunity {
  pub community_id: CommunityId,
  pub deleted: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Edit a community.
pub struct EditCommunity {
  pub community_id: CommunityId,
  /// The unique name.
  pub name: Option<String>,
  /// A longer title.
  pub title: Option<String>,
  /// A sidebar for the community in markdown.
  pub sidebar: Option<String>,
  /// A shorter, one line description of your community.
  pub description: Option<String>,
  /// Whether its an NSFW community.
  pub self_promotion: Option<bool>,
  /// Whether to restrict posting only to moderators.
  pub posting_restricted_to_mods: Option<bool>,
  pub discussion_languages: Option<Vec<LanguageId>>,
  pub visibility: Option<CommunityVisibility>,
  /// Whether its new or not.
  pub is_new: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Get a community. Must provide either an id, or a name.
pub struct GetCommunity {
  pub id: Option<CommunityId>,
  /// Example: star_trek , or star_trek@xyz.tld
  pub name: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The community response.
#[serde(rename_all = "camelCase")]
pub struct GetCommunityResponse {
  pub community_view: CommunityView,
  pub site: Option<Site>,
  pub discussion_languages: Vec<LanguageId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Fetches a random community
pub struct GetRandomCommunity {
  pub type_: Option<ListingType>,
  pub self_promotion: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Hide a community from the main view.
pub struct HideCommunity {
  pub community_id: CommunityId,
  pub hidden: bool,
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Fetches a list of communities.
pub struct ListCommunities {
  pub type_: Option<ListingType>,
  pub sort: Option<CommunitySortType>,
  /// Filter to within a given time range, in seconds.
  /// IE 60 would give results for the past minute.
  pub time_range_seconds: Option<i32>,
  pub max_depth: Option<i32>,
  pub self_promotion: Option<bool>,
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The response for listing communities.
#[serde(rename_all = "camelCase")]
pub struct ListCommunitiesResponse {
  pub communities: Vec<CommunityView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The response for listing communities.
#[serde(rename_all = "camelCase")]
pub struct ListCommunitiesTreeResponse {
  pub communities: Vec<CommunityNodeView>,
  pub count: i32,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Purges a community from the database. This will delete all content attached to that community.
#[serde(rename_all = "camelCase")]
pub struct PurgeCommunity {
  pub community_id: CommunityId,
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Remove a community
#[serde(rename_all = "camelCase")]
pub struct RemoveCommunity {
  pub community_id: CommunityId,
  pub removed: bool,
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Update a community tag.
pub struct UpdateCommunityTag {
  pub tag_id: TagId,
  pub display_name: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Delete a community tag.
pub struct DeleteCommunityTag {
  pub tag_id: TagId,
}