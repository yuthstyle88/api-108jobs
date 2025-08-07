use actix_web::web;

use actix_web::web::Json;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::newtypes::AddressId;
use lemmy_db_schema::source::address::AddressUpdateForm;
use lemmy_db_schema::{
  source::address::{Address, AddressForm},
  traits::Crud,
};
use lemmy_db_views_address::{AddressView, api::AddressResponse};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

/// Update an existing address for the user
#[tracing::instrument(skip(context))]
pub async fn update_address(
  data: Json<AddressForm>,
  context: web::Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<AddressResponse>> {
  let address_id = local_user_view.person.address_id.unwrap_or(AddressId(0));
  
  // Create the update form
  let update_form:AddressUpdateForm =  data.into_inner().try_into()?;
  
  // Update the address
  let address = Address::update(&mut context.pool(), address_id, &update_form).await?;
  let address_view = AddressView { address };
  
  Ok(Json(AddressResponse { address_view }))
}