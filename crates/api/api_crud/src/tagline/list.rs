use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_schema::source::tagline::Tagline;
use app_108jobs_db_views_site::api::{ListTaglines, ListTaglinesResponse};
use app_108jobs_utils::error::FastJobError;

pub async fn list_taglines(
  data: Query<ListTaglines>,
  context: Data<FastJobContext>,
) -> Result<Json<ListTaglinesResponse>, FastJobError> {
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(Tagline::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let taglines =
    Tagline::list(&mut context.pool(), cursor_data, data.page_back, data.limit).await?;

  let next_page = taglines.last().map(Tagline::to_cursor);

  let prev_page = taglines.first().map(Tagline::to_cursor);

  Ok(Json(ListTaglinesResponse {
    taglines,
    next_page,
    prev_page,
  }))
}
