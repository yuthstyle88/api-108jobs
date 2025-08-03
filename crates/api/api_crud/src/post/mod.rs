use chrono::{DateTime, TimeZone, Utc};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::local_site::LocalSite;
use lemmy_db_schema::source::local_user::LocalUser;
use lemmy_db_schema::source::post::Post;
use lemmy_db_schema_file::enums::PostSortType;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};

pub mod create;
pub mod delete;
pub mod read;
pub mod remove;
pub mod update;
async fn convert_published_time(
  scheduled_publish_time: Option<i64>,
  local_user_view: &LocalUserView,
  context: &FastJobContext,
) -> FastJobResult<Option<DateTime<Utc>>> {
  const MAX_SCHEDULED_POSTS: i64 = 10;
  if let Some(scheduled_publish_time) = scheduled_publish_time {
    let converted = Utc
      .timestamp_opt(scheduled_publish_time, 0)
      .single()
      .ok_or(FastJobErrorType::InvalidUnixTime)?;
    if converted < Utc::now() {
      Err(FastJobErrorType::PostScheduleTimeMustBeInFuture)?;
    }
    if !local_user_view.local_user.admin {
      let count =
        Post::user_scheduled_post_count(local_user_view.person.id, &mut context.pool()).await?;
      if count >= MAX_SCHEDULED_POSTS {
        Err(FastJobErrorType::TooManyScheduledPosts)?;
      }
    }
    Ok(Some(converted))
  } else {
    Ok(None)
  }
}

/// Returns a default instance-level post sort type, if none is given by the user.
/// Order is type, local user default, then site default.
fn post_sort_type_with_default(
  type_: Option<PostSortType>,
  local_user: Option<&LocalUser>,
  local_site: &LocalSite,
) -> PostSortType {
  type_.unwrap_or(
    local_user
        .map(|u| u.default_post_sort_type)
        .unwrap_or(local_site.default_post_sort_type),
  )
}
