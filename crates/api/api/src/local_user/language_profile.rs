use actix_web::web::{Data, Json};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::language_profile::{
  LanguageProfile,
  SaveLanguageProfiles, ListLanguageProfilesResponse, LanguageProfileItem,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResponse {
  pub success: bool,
  pub message: String,
}

pub async fn save_language_profiles(
  data: Json<SaveLanguageProfiles>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListLanguageProfilesResponse>> {
  let person_id = local_user_view.person.id;

  // Convert LanguageProfileRequest to LanguageProfileItem
  let language_profile_items: Vec<LanguageProfileItem> = data.language_profiles.iter().map(|lp| {
    LanguageProfileItem {
      id: lp.id,
      lang: Some(lp.lang.clone()),
      level_name: Some(lp.level_name.clone()),
    }
  }).collect();

  // Use the database implementation that handles duplicate validation
  let language_profiles = LanguageProfile::save_language_profile_list(
    &mut context.pool(),
    person_id,
    &language_profile_items,
  ).await?;

  Ok(Json(ListLanguageProfilesResponse {
    language_profiles,
  }))
}

pub async fn list_language_profiles(
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListLanguageProfilesResponse>> {
  let person_id = local_user_view.person.id;

  let language_profiles = LanguageProfile::list_for_person(&mut context.pool(), person_id).await.unwrap_or_else(|_| Vec::new());

  Ok(Json(ListLanguageProfilesResponse {
    language_profiles,
  }))
}
