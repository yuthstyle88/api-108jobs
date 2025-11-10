use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{
  context::FastJobContext,
  utils::check_private_instance,
};
use lemmy_db_schema::source::actor_language::CategoryLanguage;
use lemmy_db_views_category::{
  api::{GetCategory, GetCategoryResponse},
  CategoryView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

pub async fn get_category(
  data: Query<GetCategory>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<GetCategoryResponse>> {
  let local_site = context.site_config().get().await?.site_view.local_site;

  if data.name.is_none() && data.id.is_none() {
    Err(FastJobErrorType::NoIdGiven)?
  }

  check_private_instance(&local_user_view, &local_site)?;

  let local_user = local_user_view.as_ref().map(|u| &u.local_user);

  let category_id = data.id.unwrap();


  let category_view = CategoryView::read(
    &mut context.pool(),
    category_id,
    local_user,
  )
  .await?;

  let category_id = category_view.category.id;
  let discussion_languages = CategoryLanguage::read(&mut context.pool(), category_id).await?;

  Ok(Json(GetCategoryResponse {
    category_view,
    site: None,
    discussion_languages,
  }))
}
