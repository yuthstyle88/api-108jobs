use lemmy_db_schema::newtypes::{PostId, LocalUserId, PostId};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CreateProposalResponse {
    pub description: String,
    pub budget: i32,
    pub working_days: i32,
    pub brief_url: Option<String>,
    pub service_id: PostId,
    pub user_id: LocalUserId,
    pub post_id: PostId,
}
