use actix_web::web::{Data, Json};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_views_community::api::CreateCategoryGroupRequest;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub async fn _create_category_group(
  _data: Json<CreateCategoryGroupRequest>,
  _context: Data<FastJobContext>,
  _local_user_view: LocalUserView,
) -> FastJobResult<Json<()>> {
  Ok(Json(()))
}
