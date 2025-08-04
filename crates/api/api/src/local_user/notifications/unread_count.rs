use actix_web::web::Data;
use actix_web::web::Json;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_views_inbox_combined::{api::GetUnreadCountResponse, InboxCombinedViewInternal};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn unread_count(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<GetUnreadCountResponse>> {
  let person_id = local_user_view.person.id;
  let local_instance_id = local_user_view.person.instance_id;
  let show_bot_accounts = local_user_view.local_user.show_bot_accounts;

  let count = InboxCombinedViewInternal::get_unread_count(
    &mut context.pool(),
    person_id,
    local_instance_id,
    show_bot_accounts,
  )
  .await?;

  Ok(Json(GetUnreadCountResponse { count }))
}
