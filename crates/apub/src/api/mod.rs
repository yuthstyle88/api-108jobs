use lemmy_db_schema::newtypes::CommunityId;
use lemmy_db_schema::source::{local_site::LocalSite, local_user::LocalUser};
use lemmy_db_schema_file::enums::{CommentSortType, ListingType, PostSortType};

pub mod list_comments;
pub mod list_person_content;
pub mod list_posts;
pub mod read_community;
pub mod search;

/// Returns default listing type, depending if the query is for frontpage or community.
fn listing_type_with_default(
  type_: Option<ListingType>,
  local_user: Option<&LocalUser>,
  local_site: &LocalSite,
  community_id: Option<CommunityId>,
) -> ListingType {
  // On frontpage use listing type from param or admin configured default
  if community_id.is_none() {
    type_.unwrap_or(
      local_user
       .map(|u| u.default_listing_type)
       .unwrap_or(local_site.default_post_listing_type),
    )
  } else {
    // inside of community show everything
    ListingType::All
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

/// Returns a default post_time_range.
/// Order is the given, then local user default, then site default.
/// If zero is given, then the output is None.


/// Returns a default instance-level comment sort type, if none is given by the user.
/// Order is type, local user default, then site default.
fn comment_sort_type_with_default(
  type_: Option<CommentSortType>,
  local_user: Option<&LocalUser>,
  local_site: &LocalSite,
) -> CommentSortType {
  type_.unwrap_or(
    local_user
      .map(|u| u.default_comment_sort_type)
      .unwrap_or(local_site.default_comment_sort_type),
  )
}
