use crate::{CategoryReportView, PostReportView, ProposalReportView};
use app_108jobs_db::newtypes::{
  CategoryId,
  CategoryReportId,
  PostId,
  PostReportId,
  ProposalId,
  ProposalReportId,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The proposal report response.
#[serde(rename_all = "camelCase")]
pub struct ProposalReportResponse {
  pub proposal_report_view: ProposalReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A category report response.
#[serde(rename_all = "camelCase")]
pub struct CategoryReportResponse {
  pub category_report_view: CategoryReportView,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Report a proposal.
pub struct CreateProposalReport {
  pub proposal_id: ProposalId,
  pub reason: String,
  pub violates_instance_rules: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Report a proposal.
#[serde(rename_all = "camelCase")]
pub struct CreateProposalReportRequest {
  pub proposal_id: ProposalId,
  pub reason: String,
  pub violates_instance_rules: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a report for a category.
pub struct CreateCategoryReport {
  pub category_id: CategoryId,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a report for a category.
#[serde(rename_all = "camelCase")]
pub struct CreateCategoryReportRequest {
  pub category_id: CategoryId,
  pub reason: String,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a post report.
pub struct CreatePostReport {
  pub post_id: PostId,
  pub reason: String,
  pub violates_instance_rules: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a post report.
#[serde(rename_all = "camelCase")]
pub struct CreatePostReportRequest {
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
/// Resolve a proposal report (only doable by mods).
#[serde(rename_all = "camelCase")]
pub struct ResolveProposalReport {
  pub report_id: ProposalReportId,
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
