use app_108jobs_api_utils::utils::is_admin;
use app_108jobs_core::error::FastJobResult;
use app_108jobs_db::enums::CategoryVisibility;
use app_108jobs_db_views_local_user::LocalUserView;

pub mod list;
pub mod update;

fn check_category_visibility_allowed(
  visibility: Option<CategoryVisibility>,
  local_user_view: &LocalUserView,
) -> FastJobResult<()> {
  use CategoryVisibility::*;
  if visibility == Some(Private) || visibility == Some(Unlisted) {
    is_admin(local_user_view)?;
  }
  Ok(())
}
