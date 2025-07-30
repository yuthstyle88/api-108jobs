use serde::{Deserialize, Serialize};
use lemmy_db_schema_file::schema::proposals;
use crate::newtypes::{CommunityId, JobPostId, LocalUserId, PostId, ProposalId};

#[derive(Queryable, Insertable, AsChangeset, Identifiable, Selectable, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[diesel(table_name = proposals)]
#[diesel(primary_key(id))]
pub struct Proposal {
    pub id: ProposalId,
    pub description: String,
    pub budget: f64,
    pub working_days: i32,
    pub brief_url: Option<String>,
    pub service_id: i32,
    pub user_id: i32,
    pub job_post_id: JobPostId,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = proposals)]
pub struct ProposalInsertForm {
    pub description: String,
    pub budget: f64,
    pub working_days: i32,
    pub brief_url: Option<String>,
    pub service_id: CommunityId,
    pub job_post_id: PostId,
    pub user_id: LocalUserId,
}

#[derive(AsChangeset, Serialize, Deserialize, Clone, Default)]
#[diesel(table_name = proposals)]
pub struct ProposalUpdateForm {
    pub description: Option<String>,
    pub budget: Option<f64>,
    pub working_days: Option<i32>,
    pub brief_url: Option<String>,
}
