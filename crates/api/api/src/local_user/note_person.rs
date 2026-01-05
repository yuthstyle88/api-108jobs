use actix_web::web::Data;
use actix_web::web::Json;
use app_108jobs_api_utils::{
  context::FastJobContext,
  utils::{get_url_blocklist, process_markdown, slur_regex},
};
use app_108jobs_db_schema::source::person::{PersonActions, PersonNoteForm};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_person::api::NotePerson;
use app_108jobs_db_views_site::api::SuccessResponse;
use app_108jobs_utils::{
  error::{FastJobErrorType, FastJobResult},
  utils::{slurs::check_slurs, validation::is_valid_body_field},
};

pub async fn user_note_person(
  data: Json<NotePerson>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let target_id = data.person_id;
  let person_id = local_user_view.person.id;

  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;

  // Don't let a person note themselves
  if target_id == person_id {
    Err(FastJobErrorType::CantNoteYourself)?
  }

  // If the note is empty, delete it
  if data.note.is_empty() {
    PersonActions::delete_note(&mut context.pool(), person_id, target_id).await?;
  } else {
    check_slurs(&data.note, &slur_regex)?;
    is_valid_body_field(&data.note, false)?;

    let note = process_markdown(&data.note, &slur_regex, &url_blocklist, &context).await?;
    let note_form = PersonNoteForm::new(person_id, target_id, note);

    PersonActions::note(&mut context.pool(), &note_form).await?;
  }

  Ok(Json(SuccessResponse::default()))
}
