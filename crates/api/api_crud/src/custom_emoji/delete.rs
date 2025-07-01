use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::FastJobContext, utils::is_admin};
use lemmy_db_schema::{source::custom_emoji::CustomEmoji, traits::Crud};
use lemmy_db_views_custom_emoji::api::DeleteCustomEmoji;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::FastJobResult;

pub async fn delete_custom_emoji(
  data: Json<DeleteCustomEmoji>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  CustomEmoji::delete(&mut context.pool(), data.id).await?;

  Ok(Json(SuccessResponse::default()))
}
