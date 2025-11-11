use crate::{CommentReportView, CategoryReportView, PostReportView};
use lemmy_db_schema::newtypes::{
    CommentId,
    CommentReportId,
    CategoryId,
    CategoryReportId,
    PostId,
    PostReportId,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The comment report response.
#[serde(rename_all = "camelCase")]
pub struct CommentReportResponse {
  pub comment_report_view: CommentReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A category report response.
#[serde(rename_all = "camelCase")]
pub struct CategoryReportResponse {
  pub category_report_view: CategoryReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Report a comment.
#[serde(rename_all = "camelCase")]
pub struct CreateCommentReport {
  pub comment_id: CommentId,
  pub reason: String,
  pub violates_instance_rules: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a report for a category.
#[serde(rename_all = "camelCase")]
pub struct CreateCategoryReport {
  pub category_id: CategoryId,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a post report.
#[serde(rename_all = "camelCase")]
pub struct CreatePostReport {
  pub post_id: PostId,
  pub reason: String,
  pub violates_instance_rules: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Get a count of the number of reports.
#[serde(rename_all = "camelCase")]
pub struct GetReportCount {
  pub category_id: Option<CategoryId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A response for the number of reports.
#[serde(rename_all = "camelCase")]
pub struct GetReportCountResponse {
  pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Resolve a comment report (only doable by mods).
#[serde(rename_all = "camelCase")]
pub struct ResolveCommentReport {
  pub report_id: CommentReportId,
  pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Resolve a category report.
#[serde(rename_all = "camelCase")]
pub struct ResolveCategoryReport {
  pub report_id: CategoryReportId,
  pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Resolve a post report (mods only).
#[serde(rename_all = "camelCase")]
pub struct ResolvePostReport {
  pub report_id: PostReportId,
  pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The post report response.
#[serde(rename_all = "camelCase")]
pub struct PostReportResponse {
  pub post_report_view: PostReportView,
}
