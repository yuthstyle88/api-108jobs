use actix_web::web;

use actix_web::web::Json;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::identity_card::{IdentityCard, IdentityCardForm};
use lemmy_db_schema::traits::Crud;
use lemmy_db_views_identity_card::{IdentityCardView, api::IdentityCardResponse};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

/// Create a new identity card for the user
#[tracing::instrument(skip(context))]
pub async fn create_identity_card(
  data: Json<IdentityCardForm>,
  context: web::Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<IdentityCardResponse>> {
  // Convert the form data to an insert form
  let form = IdentityCardForm {
    local_user_id: local_user_view.local_user.id,
    ..data.into_inner()
  }.try_into()?;
  // Insert the identity card into the database
  let identity_card = IdentityCard::create(&mut context.pool(), &form).await?;
  
  // Create an identity card view from the identity card
  let identity_card_view = IdentityCardView { identity_card };
  
  // Return the identity card response
  Ok(Json(IdentityCardResponse { identity_card_view }))
}
