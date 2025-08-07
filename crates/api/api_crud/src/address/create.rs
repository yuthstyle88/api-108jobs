use actix_web::web;

use actix_web::web::Json;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::address::{Address, AddressForm};
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_address::{AddressView, api::AddressResponse};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

/// Create a new address for the user
#[tracing::instrument(skip(context))]
pub async fn create_address(
  data: Json<AddressForm>,
  context: web::Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<AddressResponse>> {
  // Convert the form data to an insert form
  let form = AddressForm {
    local_user_id: local_user_view.local_user.id,
    ..data.into_inner()
  }.try_into()?;
  // Insert the address into the database
  let address = Address::create(&mut context.pool(), &form).await?;
  
  // Create an address view from the address
  let address_view = AddressView { address };
  
  // Return the address response
  Ok(Json(AddressResponse { address_view }))
}
