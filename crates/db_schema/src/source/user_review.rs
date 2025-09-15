use crate::newtypes::{PersonId, UserReviewId, WorkflowId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::user_review;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = user_review))]
#[cfg_attr(feature = "full", diesel(primary_key(id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[serde(rename_all = "camelCase")]
pub struct UserReview {
  pub id: UserReviewId,
  pub reviewer_id: PersonId,
  pub reviewee_id: PersonId,
  pub workflow_id: WorkflowId,
  pub rating: i16,
  pub comment: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = user_review))]
pub struct UserReviewInsertForm {
  pub reviewer_id: PersonId,
  pub reviewee_id: PersonId,
  pub workflow_id: WorkflowId,
  pub rating: i16,
  #[new(default)]
  pub comment: Option<String>,
  #[new(default)]
  pub created_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub updated_at: Option<Option<DateTime<Utc>>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = user_review))]
pub struct UserReviewUpdateForm {
  pub rating: Option<i16>,
  pub comment: Option<Option<String>>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
}
