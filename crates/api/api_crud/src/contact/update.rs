use actix_web::web;

use actix_web::web::Json;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::contact::ContactUpdateForm;
use lemmy_db_schema::{
  source::contact::{Contact},
  traits::Crud,
};
use lemmy_db_views_contact::{ContactView, api::ContactResponse};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

use lemmy_db_schema::source::contact::ContactForm;

/// Update an existing contact for the user
#[tracing::instrument(skip(context))]
pub async fn update_contact(
  data: Json<ContactForm>,
  context: web::Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ContactResponse>> {
  let contact_id = local_user_view.person.contact_id;
  
  // Create the update form
  let update_form: ContactUpdateForm = data.into_inner().try_into()?;
  
  // Update the contact
  let contact = Contact::update(&mut context.pool(), contact_id, &update_form).await?;
  let contact_view = ContactView { contact };
  
  Ok(Json(ContactResponse { contact_view }))
}