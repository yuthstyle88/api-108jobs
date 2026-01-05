use crate::newtypes::{PersonId, UserReviewId, WorkflowId};
#[cfg(feature = "full")]
use crate::{
  source::user_review::{UserReview, UserReviewInsertForm, UserReviewUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
#[cfg(feature = "full")]
use diesel::QueryDsl;
use diesel::{ExpressionMethods, OptionalExtension};
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;
#[cfg(feature = "full")]
use app_108jobs_db_schema_file::schema::user_review;
use app_108jobs_db_schema_file::schema::user_review::dsl as ur;
#[cfg(feature = "full")]
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use chrono::Utc;

#[cfg(feature = "full")]
impl Crud for UserReview {
  type InsertForm = UserReviewInsertForm;
  type UpdateForm = UserReviewUpdateForm;
  type IdType = UserReviewId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(user_review::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(user_review::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}

#[cfg(feature = "full")]
impl UserReview {
  pub async fn get_by_triplet(
    pool: &mut DbPool<'_>,
    reviewer_id: PersonId,
    reviewee_id: PersonId,
    workflow_id: WorkflowId,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    ur::user_review
      .filter(ur::reviewer_id.eq(reviewer_id))
      .filter(ur::reviewee_id.eq(reviewee_id))
      .filter(ur::workflow_id.eq(workflow_id))
      .first::<Self>(conn)
      .await
      .optional()
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn upsert_by_triplet(
    pool: &mut DbPool<'_>,
    reviewer_id: PersonId,
    reviewee_id: PersonId,
    workflow_id: WorkflowId,
    rating: i16,
    comment: Option<String>,
  ) -> FastJobResult<Self> {
    if let Some(existing) = Self::get_by_triplet(pool, reviewer_id, reviewee_id, workflow_id).await? {
      let form = UserReviewUpdateForm {
        rating: Some(rating),
        comment: Some(comment),
        updated_at: Some(Some(Utc::now())),
      };
      return Self::update(pool, existing.id, &form).await;
    }
    let form = UserReviewInsertForm {
      reviewer_id,
      reviewee_id,
      workflow_id,
      rating,
      comment,
      created_at: Some(Utc::now()),
      updated_at: Some(None),
    };
    Self::create(pool, &form).await
  }

  pub async fn list_by_reviewee(
    pool: &mut DbPool<'_>,
    reviewee_id: PersonId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    ur::user_review
      .filter(ur::reviewee_id.eq(reviewee_id))
      .order(ur::created_at.desc())
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}
