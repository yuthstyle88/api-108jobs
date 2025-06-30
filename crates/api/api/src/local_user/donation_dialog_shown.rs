use actix_web::web::{Data, Json};
use chrono::Utc;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::local_user::{LocalUser, LocalUserUpdateForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::FastJobResult;

pub async fn donation_dialog_shown(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let form = LocalUserUpdateForm {
    last_donation_notification_at: Some(Utc::now()),
    ..Default::default()
  };
  LocalUser::update(&mut context.pool(), local_user_view.local_user.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}
