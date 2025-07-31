use chrono::Utc;
use lemmy_db_schema::newtypes::{CommunityId, ProposalId};
use lemmy_db_schema::source::proposal::Proposal;
use lemmy_db_schema_file::schema::proposals;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// Response for a single proposal
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct ProposalResponse {
  pub proposal: Proposal,
}

// Delete a proposal
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct DeleteProposal {
  pub id: ProposalId,
}

// Edit a proposal
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, diesel::AsChangeset)]
#[diesel(table_name = proposals)]
pub struct EditProposal {
  pub id: ProposalId,
  pub description: Option<String>,
  pub budget: Option<f64>,
  pub working_days: Option<i32>,
  pub brief_url: Option<String>,
}

// List proposals with pagination

// Response for listing proposals

#[derive(Debug, Deserialize)]
pub struct ListProposalRequest {
  pub post_id: Option<i32>,
  pub user_id: Option<i32>,
  pub service_id: Option<i32>,
  pub limit: Option<i64>,
  pub offset: Option<i64>,
  pub sort: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListProposalResponse {
  pub proposals: Vec<Proposal>,
  pub total: i64,
}
