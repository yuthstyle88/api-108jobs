use actix_web::web::*;
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::error::FastJobResult;

pub mod delete;
pub mod download;
pub mod upload;
mod utils;

pub async fn pictrs_health(context: Data<FastJobContext>) -> FastJobResult<Json<SuccessResponse>> {
  let pictrs_config = context.settings().pictrs()?;
  let url = format!("{}healthz", pictrs_config.url);

  context
    .pictrs_client()
    .get(url)
    .send()
    .await?
    .error_for_status()?;

  Ok(Json(SuccessResponse::default()))
}
