use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::user_review::UserReview;
use lemmy_db_schema::source::workflow::Workflow;
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_user_review::api::{
  ListUserReviewsQuery, ListUserReviewsResponse, SubmitUserReviewForm, SubmitUserReviewResponse,
  ValidSubmitUserReview,
};
use lemmy_db_views_user_review::UserReviewView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

pub async fn submit_user_review(
  data: Json<SubmitUserReviewForm>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SubmitUserReviewResponse>> {
  let validated: ValidSubmitUserReview = match data.into_inner().try_into() {
    Ok(v) => v,
    Err(msg) => {
      return Err(FastJobErrorType::InvalidField(msg).into());
    }
  };
  let reviewer_person_id = local_user_view.person.id;
  let reviewee_id = validated.0.reviewee_id;
  let workflow_id = validated.0.workflow_id;
  let rating = validated.0.rating;
  let comment = validated.0.comment;

  // Ensure the workflow exists
  if !Workflow::exists(&mut context.pool(), workflow_id).await? {
    Err(FastJobErrorType::WorkflowDoesNotExist)?;
  }

  // Upsert review based on (reviewer, reviewee, workflow)
  let review = UserReview::upsert_by_triplet(
    &mut context.pool(),
    reviewer_person_id,
    reviewee_id,
    workflow_id,
    rating,
    comment,
  )
  .await?;

  Ok(Json(SubmitUserReviewResponse { review }))
}

pub async fn list_user_reviews(
  data: Query<ListUserReviewsQuery>,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<ListUserReviewsResponse>> {
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(UserReviewView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  // Sanitize limit: default 20, clamp to 1..=100
  let mut lim = data.limit.unwrap_or(20);
  if lim <= 0 {
    lim = 20;
  }
  if lim > 100 {
    lim = 100;
  }
  let lim = Some(lim);

  // If a cursor exists and direction not specified, default to paging backward (older)
  let effective_page_back = match (cursor_data.as_ref(), data.page_back) {
    (Some(_), None) => Some(true),
    _ => data.page_back,
  };

  let results = UserReviewView::list_for_user(
    &mut context.pool(),
    data.profile_id,
    lim,
    cursor_data,
    effective_page_back,
  )
  .await?;

  let next_page = results.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = results.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListUserReviewsResponse {
    reviews: results,
    next_page,
    prev_page,
  }))
}
