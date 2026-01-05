use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::{context::FastJobContext, utils::check_private_instance};
use app_108jobs_db_schema::source::{actor_language::CategoryLanguage, category::Category};
use app_108jobs_db_views_category::{
  api::{CategoryResponse, GetRandomCategory},
  CategoryView,
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_utils::error::FastJobResult;

pub async fn get_random_category(
  data: Query<GetRandomCategory>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<CategoryResponse>> {
  let site_view = context.site_config().get().await?.site_view;

  check_private_instance(&local_user_view, &site_view.local_site)?;

  let local_user = local_user_view.as_ref().map(|u| &u.local_user);

  let random_category_id =
    Category::get_random_category_id(&mut context.pool(), &data.type_, data.self_promotion)
      .await?;

  let category_view =
    CategoryView::read(&mut context.pool(), random_category_id, local_user).await?;

  let discussion_languages =
    CategoryLanguage::read(&mut context.pool(), random_category_id).await?;

  Ok(Json(CategoryResponse {
    category_view,
    discussion_languages,
  }))
}
