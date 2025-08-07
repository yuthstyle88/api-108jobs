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
  let address_id = local_user_view.person.address_id;
  
  // Get the address view from the database
  let address_view = AddressView::find_by_id(
    &mut context.pool(),
    address_id,
  ).await?;
  // Return the address response
  Ok(Json(AddressResponse { address_view }))
}
