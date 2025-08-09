use actix_web::web;

use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::address::AddressUpdateForm;
use lemmy_db_schema::{source::address::Address, traits::Crud};
use lemmy_db_views_address::api::{UpsertAddress, UpsertAddressRequest};
use lemmy_db_views_address::{api::AddressResponse, AddressView};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

/// Update an existing address for the user
#[tracing::instrument(skip(context))]
pub async fn update_address(
  data: Json<UpsertAddressRequest>,
  context: web::Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<AddressResponse>> {
  let address_id = local_user_view.person.address_id;
  let data: UpsertAddress = data.into_inner().try_into()?;

  let update_form = AddressUpdateForm {
    address_line1: Some(data.address_line1),
    address_line2: Some(data.address_line2),
    subdistrict: Some(Some(data.subdistrict)),
    district: Some(data.district),
    province: Some(data.province),
    postal_code: Some(data.postal_code),
    country_id: Some(data.country_id),
    updated_at: Some(Some(Utc::now())),
    ..Default::default()
  };

  // Update the address
  let address = Address::update(&mut context.pool(), address_id, &update_form).await?;
  let address_view = AddressView { address };

  Ok(Json(AddressResponse { address_view }))
}
