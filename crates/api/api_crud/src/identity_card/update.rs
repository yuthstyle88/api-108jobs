use actix_web::web;

use actix_web::web::Json;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::identity_card::IdentityCardUpdateForm;
use lemmy_db_schema::{
  source::identity_card::{IdentityCard, IdentityCardForm},
  traits::Crud,
};
use lemmy_db_views_identity_card::{IdentityCardView, api::IdentityCardResponse};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

/// Update an existing identity_card for the user
#[tracing::instrument(skip(context))]
pub async fn update_identity_card(
  data: Json<IdentityCardForm>,
  context: web::Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<IdentityCardResponse>> {
  let identity_card_id = local_user_view.person.identity_card_id;
  // Create the update form
  let update_form: IdentityCardUpdateForm =  data.into_inner().try_into()?;
  
  // Update the identity_card
  let identity_card = IdentityCard::update(&mut context.pool(), identity_card_id, &update_form).await?;
  let identity_card_view = IdentityCardView { identity_card };
  
  Ok(Json(IdentityCardResponse { identity_card_view }))
}