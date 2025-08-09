use actix_web::web;

use actix_web::web::Json;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::identity_card::IdentityCardUpdateForm;
use lemmy_db_schema::{
  source::identity_card::IdentityCard,
  traits::Crud,
};
use lemmy_db_views_identity_card::api::{UpsertIDCard, UpsertIDCardRequest};
use lemmy_db_views_identity_card::{api::IdentityCardResponse, IdentityCardView};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

/// Update an existing identity_card for the user
#[tracing::instrument(skip(context))]
pub async fn update_identity_card(
  data: Json<UpsertIDCardRequest>,
  context: web::Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<IdentityCardResponse>> {
  let identity_card_id = local_user_view.person.identity_card_id;

  let data: UpsertIDCard = data.into_inner().try_into()?;
  
  IdentityCard::check_id_number_exist(&mut context.pool(), &identity_card_id, &data.id_number).await?;

  let update_form = IdentityCardUpdateForm {
    address_id: Some(data.address_id),
    id_number: Some(data.id_number),
    issued_date: Some(data.issued_date),
    expiry_date: Some(data.expiry_date),
    full_name: Some(data.full_name),
    date_of_birth: Some(data.date_of_birth),
    nationality: Some(data.nationality),
    ..Default::default()
  };

  // Update the identity_card
  let identity_card =
    IdentityCard::update(&mut context.pool(), identity_card_id, &update_form).await?;
  let identity_card_view = IdentityCardView { identity_card };

  Ok(Json(IdentityCardResponse { identity_card_view }))
}
