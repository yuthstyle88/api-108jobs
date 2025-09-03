use actix_web::web::{Data, Json, Path};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::person::{Person, PersonUpdateForm, SaveUserProfileForm};

use lemmy_db_schema::traits::Crud;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::FastJobResult;
use serde::{Deserialize, Serialize};
use lemmy_db_views_person::api::VisitProfileResponse;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileResponse;

pub async fn save_profile(
  data: Json<SaveUserProfileForm>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<SuccessResponse>> {
  let person_id = local_user_view.person.id;

  let person_form = data.clone().person;
  let form = PersonUpdateForm {
    display_name: Some(person_form.display_name.clone()),
    name: person_form.name.clone(),
    avatar: Some(person_form.avatar.clone()),
    bio: Some(person_form.bio.clone()),
    skills: Some(person_form.skills),
    ..Default::default()
  };
  let _person = Person::update(&mut context.pool(), person_id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}

pub async fn visit_profile(
  context: Data<FastJobContext>,
  username: Path<String>,
) -> FastJobResult<Json<VisitProfileResponse>> {
  let found_person = Person::read_by_name_or_local_user_id(&mut context.pool(), &username).await?;

  Ok(Json(VisitProfileResponse {
    profile: found_person,
  }))
}
