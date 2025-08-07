use actix_web::web::{Data, Json};
use lemmy_api_utils::{
  context::FastJobContext,
};

use lemmy_db_views_contact::{ContactView, api::ContactResponse};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn get_contact(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ContactResponse>> {
  let local_user_id = local_user_view.local_user.id;
  // Get the contact view from the database
  let contact_view = ContactView::find_by_local_user_id(
    &mut context.pool(),
    local_user_id,
  ).await?;
  
  // Return the contact response
  Ok(Json(ContactResponse { contact_view }))
}
