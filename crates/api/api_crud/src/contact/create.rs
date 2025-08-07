use actix_web::web;

use actix_web::web::Json;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::contact::ContactForm;
use lemmy_db_schema::{
  source::contact::{Contact},
  traits::Crud,
};
use lemmy_db_views_contact::{ContactView, api::ContactResponse};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

/// Create a new contact for the user
#[tracing::instrument(skip(context))]
pub async fn create_contact(
  data: Json<ContactForm>,
  context: web::Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ContactResponse>> {
  let form = ContactForm {
    local_user_id: local_user_view.local_user.id,
    ..data.into_inner()
  }.try_into()?;
  // Create the contact
  let contact = Contact::create(&mut context.pool(), &form).await?;
  let contact_view = ContactView { contact };
  Ok(Json(ContactResponse { contact_view }))
}
