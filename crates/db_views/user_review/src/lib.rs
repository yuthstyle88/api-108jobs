use diesel::{Queryable, Selectable};
use app_108jobs_db_schema::source::person::Person;
use app_108jobs_db_schema::source::user_review::UserReview;
use app_108jobs_db_schema::source::workflow::Workflow;
use serde::{Deserialize, Serialize};

pub mod api;

#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[serde(rename_all = "camelCase")]
/// A user review view, including sender and room.
pub struct UserReviewView {
    #[cfg_attr(feature = "full", diesel(embed))]
    pub review: UserReview,
    #[cfg_attr(feature = "full", diesel(embed))]
    pub reviewer: Person,
    #[cfg_attr(feature = "full", diesel(embed))]
    pub reviewee: Person,
    #[cfg_attr(feature = "full", diesel(embed))]
    pub workflow: Workflow,
}
