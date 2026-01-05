use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::traits::PaginationCursorBuilder;
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_person_liked_combined::{
  impls::PersonLikedCombinedQuery,
  ListPersonLiked,
  ListPersonLikedResponse,
  PersonLikedCombinedView,
};
use app_108jobs_utils::error::FastJobResult;

pub async fn list_person_liked(
  data: Query<ListPersonLiked>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListPersonLikedResponse>> {
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(PersonLikedCombinedView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let liked = PersonLikedCombinedQuery {
    type_: data.type_,
    like_type: data.like_type,
    cursor_data,
    page_back: data.page_back,
    limit: data.limit,
    no_limit: None,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  let next_page = liked.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = liked.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListPersonLikedResponse {
    liked,
    next_page,
    prev_page,
  }))
}
