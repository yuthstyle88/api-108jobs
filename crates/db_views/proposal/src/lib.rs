use diesel::internal::derives::multiconnection::chrono;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
};
use lemmy_db_schema::newtypes::{CommunityId, LocalUserId, PostId, ProposalId};
use lemmy_db_schema::source::proposal::Proposal;

#[cfg(feature = "full")]
pub mod impls;
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets your saved posts and comments
#[serde(rename_all = "camelCase")]
pub struct CreateProposalRequest {
  pub description: String,
  pub budget: f64,
  pub working_days: i32,
  pub brief_url: Option<String>,
  pub post_id: PostId,
  pub community_id: CommunityId
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets your saved posts and comments
#[serde(rename_all = "camelCase")]
pub struct CreateProposalResponse {
  pub id: ProposalId,
  pub description: String,
  pub budget: f64,
  pub working_days: i32,
  pub brief_url: Option<String>,
  pub user_id: LocalUserId,
  pub post_id: PostId,
  pub community_id: CommunityId,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets your saved posts and comments
#[serde(rename_all = "camelCase")]
pub struct DeleteProposalRequest {
  pub id: ProposalId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets your saved posts and comments
#[serde(rename_all = "camelCase")]
pub struct MyProposalsQuery {
  pub page: Option<u64>,
  pub page_size: Option<u64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets your saved posts and comments
#[serde(rename_all = "camelCase")]
pub struct MyProposalsResponse {
  pub proposals: Vec<Proposal>,
  pub total_items: u64,
  pub page: u64,
  pub page_size: u64,
  pub total_pages: u64,
}

