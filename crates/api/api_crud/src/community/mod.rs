use lemmy_api_utils::utils::is_admin;
use lemmy_db_schema_file::enums::CommunityVisibility;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;

pub mod delete;
pub mod list;
pub mod remove;
pub mod update;

/// For now only admins can make communities private or hidden, in order to
/// prevent abuse. Need to implement admin approval for new communities to
/// get rid of this.
fn check_community_visibility_allowed(
  visibility: Option<CommunityVisibility>,
  local_user_view: &LocalUserView,
) -> FastJobResult<()> {
  use CommunityVisibility::*;
  if visibility == Some(Private) || visibility == Some(Unlisted) {
    is_admin(local_user_view)?;
  }
  Ok(())
}
