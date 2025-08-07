use actix_web::web::{Data, Json};
use lemmy_api_utils::{
  context::FastJobContext,
};
use lemmy_db_views_address::AddressView;
use lemmy_db_views_address::api::AddressResponse;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn get_address(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<AddressResponse>> {
  let local_user_id = local_user_view.local_user.id;
  
  // Get the address view from the database
  let address_view = AddressView::find_by_local_user_id(
    &mut context.pool(),
    local_user_id,
  ).await?;
  // Return the address response
  Ok(Json(AddressResponse { address_view }))
}
