use super::check_category_visibility_allowed;
use actix_web::web::Data;
use actix_web::web::Json;
use chrono::Utc;
use app_108jobs_api_utils::utils::{check_category_deleted_removed, is_admin};
use app_108jobs_api_utils::{
  build_response::build_category_response,
  context::FastJobContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_self_promotion_allowed, get_url_blocklist, process_markdown_opt, slur_regex},
};
use app_108jobs_db_schema::{
  source::{
      actor_language::{CategoryLanguage, SiteLanguage},
      category::{Category, CategoryUpdateForm},
  },
  traits::Crud,
  utils::diesel_string_update,
};
use app_108jobs_db_views_category::api::{CategoryResponse, EditCategory};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_utils::{
  error::{FastJobErrorType, FastJobResult},
  utils::{slurs::check_slurs_opt, validation::is_valid_body_field},
};

pub async fn update_category(
  data: Json<EditCategory>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<CategoryResponse>> {
  is_admin(&local_user_view)?;

  let local_site = context.site_config().get().await?.site_view.local_site;

  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  check_slurs_opt(&data.title, &slur_regex)?;
  check_self_promotion_allowed(data.self_promotion, Some(&local_site))?;

  let sidebar = diesel_string_update(
    process_markdown_opt(&data.sidebar, &slur_regex, &url_blocklist, &context)
      .await?
      .as_deref(),
  );

  if let Some(Some(sidebar)) = &sidebar {
    is_valid_body_field(sidebar, false)?;
  }

  check_category_visibility_allowed(data.visibility, &local_user_view)?;
  let description = diesel_string_update(data.description.as_deref());

  let old_category = Category::read(&mut context.pool(), data.category_id).await?;

  check_category_deleted_removed(&old_category)?;

  let category_id = data.category_id;
  if let Some(languages) = data.discussion_languages.clone() {
    let site_languages = SiteLanguage::read_local_raw(&mut context.pool()).await?;
    // check that category languages are a subset of site languages
    // https://stackoverflow.com/a/64227550
    let is_subset = languages.iter().all(|item| site_languages.contains(item));
    if !is_subset {
      Err(FastJobErrorType::LanguageNotAllowed)?
    }
    CategoryLanguage::update(&mut context.pool(), languages, category_id).await?;
  }

  let category_form = CategoryUpdateForm {
    name: data.name.clone(),
    title: data.title.clone(),
    sidebar,
    description,
    self_promotion: data.self_promotion,
    posting_restricted_to_mods: data.posting_restricted_to_mods,
    visibility: data.visibility,
    updated_at: Some(Some(Utc::now())),
    is_new: data.is_new,
    ..Default::default()
  };

  let category_id = data.category_id;
  let category = Category::update(&mut context.pool(), category_id, &category_form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::UpdateCategory(local_user_view.person.clone(), category),
    &context,
  )?;

  build_category_response(&context, local_user_view, category_id).await
}
