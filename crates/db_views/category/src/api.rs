use crate::{CategoryNodeView, CategoryView};
use lemmy_db_schema::{
    newtypes::{CategoryId, LanguageId, PaginationCursor, TagId},
    source::site::Site,
    CategorySortType,
};
use lemmy_db_schema_file::enums::{CategoryVisibility, ListingType};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Parameter for setting category icon or banner. Can't use POST data here as it already contains
/// the image data.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct CategoryIdQuery {
  pub id: CategoryId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A simple category response.
#[serde(rename_all = "camelCase")]
pub struct CategoryResponse {
  pub category_view: CategoryView,
  pub discussion_languages: Vec<LanguageId>,
}

#[skip_serializing_none]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
/// Create a category.
#[serde(rename_all = "camelCase")]
pub struct CreateCategory {
  /// The unique name.
  pub name: String,
  /// A longer title (can be seen as subname)
  pub title: String,
  /// A sidebar for the category in markdown.
  pub sidebar: Option<String>,
  /// A shorter, one line description of your category.
  pub description: Option<String>,
  /// An icon URL.
  pub icon: Option<String>,
  /// A banner URL.
  pub banner: Option<String>,
  /// Whether its self-promotion category.
  pub self_promotion: Option<bool>,
  /// Whether to restrict posting only to moderators.
  pub posting_restricted_to_mods: Option<bool>,
  pub discussion_languages: Option<Vec<LanguageId>>,
  pub visibility: Option<CategoryVisibility>,
  pub is_new: Option<bool>,
  pub parent_id: Option<CategoryId>,
}

#[skip_serializing_none]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
/// Create a category (this is the replacement of a category and subcategory of a job).
pub struct CreateCategoryRequest {
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
  pub parent_id: Option<CategoryId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Create a tag for a category.
pub struct CreateCategoryTag {
  pub category_id: CategoryId,
  pub display_name: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Delete your own category.
pub struct DeleteCategory {
  pub category_id: CategoryId,
  pub deleted: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Edit a category.
pub struct EditCategory {
  pub category_id: CategoryId,
  /// The unique name.
  pub name: Option<String>,
  /// A longer title.
  pub title: Option<String>,
  /// A sidebar for the category in markdown.
  pub sidebar: Option<String>,
  /// A shorter, one line description of your category.
  pub description: Option<String>,
  /// Whether its an NSFW category.
  pub self_promotion: Option<bool>,
  /// Whether to restrict posting only to moderators.
  pub posting_restricted_to_mods: Option<bool>,
  pub discussion_languages: Option<Vec<LanguageId>>,
  pub visibility: Option<CategoryVisibility>,
  /// Whether its new or not.
  pub is_new: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Get a category. Must provide either an id, or a name.
pub struct GetCategory {
  pub id: Option<CategoryId>,
  /// Example: star_trek , or star_trek@xyz.tld
  pub name: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The category response.
#[serde(rename_all = "camelCase")]
pub struct GetCategoryResponse {
  pub category_view: CategoryView,
  pub site: Option<Site>,
  pub discussion_languages: Vec<LanguageId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Fetches a random category
pub struct GetRandomCategory {
  pub type_: Option<ListingType>,
  pub self_promotion: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Hide a category from the main view.
pub struct HideCategory {
  pub category_id: CategoryId,
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
  pub sort: Option<CategorySortType>,
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
  pub communities: Vec<CategoryView>,
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
  pub communities: Vec<CategoryNodeView>,
  pub count: i32,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Purges a category from the database. This will delete all content attached to that category.
#[serde(rename_all = "camelCase")]
pub struct PurgeCategory {
  pub category_id: CategoryId,
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Remove a category
#[serde(rename_all = "camelCase")]
pub struct RemoveCategory {
  pub category_id: CategoryId,
  pub removed: bool,
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Update a category tag.
pub struct UpdateCategoryTag {
  pub tag_id: TagId,
  pub display_name: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Delete a category tag.
pub struct DeleteCategoryTag {
  pub tag_id: TagId,
}