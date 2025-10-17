use crate::UserReviewView;
use lemmy_db_schema::newtypes::{PaginationCursor, PersonId, WorkflowId};
use lemmy_db_schema::source::user_review::UserReview;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct SubmitUserReviewForm {
    pub reviewee_id: PersonId,
    pub workflow_id: WorkflowId,
    pub rating: i16,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidSubmitUserReview(pub SubmitUserReviewForm);

impl TryFrom<SubmitUserReviewForm> for ValidSubmitUserReview {
    type Error = String;
    fn try_from(value: SubmitUserReviewForm) -> Result<Self, Self::Error> {
        if value.rating <= 0 && value.rating > 5 {
            return Err("Invalid rating".to_string());
        }
        Ok(ValidSubmitUserReview(value))
    }
}


#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
pub struct SubmitUserReviewResponse {
    pub review: UserReview,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// Fetches a list of User Reviews.
pub struct ListUserReviewsQuery {
    pub profile_id: PersonId,
    pub page_cursor: Option<PaginationCursor>,
    pub page_back: Option<bool>,
    pub limit: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The response for listing user reviews.
#[serde(rename_all = "camelCase")]
pub struct ListUserReviewsResponse {
    pub reviews: Vec<UserReviewView>,
    /// the pagination cursor to use to fetch the next page
    pub next_page: Option<PaginationCursor>,
    pub prev_page: Option<PaginationCursor>,
}
