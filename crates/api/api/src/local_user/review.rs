use actix_web::web::{Data, Json, Path};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::{PersonId, WorkflowId};
use lemmy_db_schema::source::user_review::UserReview;
use lemmy_db_schema::source::workflow::Workflow;
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SubmitUserReviewForm {
  pub reviewee_id: i32,
  pub workflow_id: i32,
  pub rating: i16,
  pub comment: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmitUserReviewResponse {
  pub review_id: i32,
  pub success: bool,
}

#[derive(Debug, Serialize)]
pub struct ListUserReviewsResponse {
  pub reviews: Vec<UserReview>,
  pub average_rating: Option<f32>,
  pub count: usize,
}

pub async fn submit_user_review(
  data: Json<SubmitUserReviewForm>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SubmitUserReviewResponse>> {
  let reviewer_person_id = local_user_view.person.id;
  let reviewee_id = PersonId(data.reviewee_id);
  let workflow_id = WorkflowId(data.workflow_id);
  let rating = data.rating;

  if !(1..=5).contains(&rating) {
    return Err(FastJobErrorType::InvalidField("rating must be between 1 and 5".into()).into());
  }

  // Ensure workflow exists
  let _wf = Workflow::read(&mut context.pool(), workflow_id).await?;

  // Upsert review based on (reviewer, reviewee, workflow)
  let review = UserReview::upsert_by_triplet(
    &mut context.pool(),
    reviewer_person_id,
    reviewee_id,
    workflow_id,
    rating,
    data.comment.clone(),
  )
  .await?;

  Ok(Json(SubmitUserReviewResponse {
    review_id: review.id.0,
    success: true,
  }))
}

pub async fn list_user_reviews(
  path: Path<i32>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<ListUserReviewsResponse>> {
  let reviewee_id = PersonId(path.into_inner());
  let reviews = UserReview::list_by_reviewee(&mut context.pool(), reviewee_id).await?;
  let count = reviews.len();
  let average_rating = if count == 0 {
    None
  } else {
    let sum: i32 = reviews.iter().map(|r| r.rating as i32).sum();
    Some(sum as f32 / count as f32)
  };

  Ok(Json(ListUserReviewsResponse {
    reviews,
    average_rating,
    count,
  }))
}
