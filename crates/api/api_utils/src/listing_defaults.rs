use app_108jobs_db::{
  enums::{ListingType, PostSortType, ProposalSortType},
  newtypes::CategoryId,
  source::{local_site::LocalSite, local_user::LocalUser},
};

/// Returns default listing type, depending if the query is for frontpage or category.
pub fn listing_type_with_default(
  type_: Option<ListingType>,
  local_user: Option<&LocalUser>,
  local_site: &LocalSite,
  category_id: Option<CategoryId>,
) -> ListingType {
  // On frontpage use listing type from param or admin configured default
  if category_id.is_none() {
    type_.unwrap_or(
      local_user
        .map(|u| u.default_listing_type)
        .unwrap_or(local_site.default_post_listing_type),
    )
  } else {
    // inside of category show everything
    ListingType::All
  }
}

/// Returns a default instance-level post sort type, if none is given by the user.
/// Order is type, local user default, then site default.
pub fn post_sort_type_with_default(
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

/// Returns a default instance-level proposal sort type, if none is given by the user.
/// Order is type, local user default, then site default.
pub fn proposal_sort_type_with_default(
  type_: Option<ProposalSortType>,
  local_user: Option<&LocalUser>,
  local_site: &LocalSite,
) -> ProposalSortType {
  type_.unwrap_or(
    local_user
      .map(|u| u.default_proposal_sort_type)
      .unwrap_or(local_site.default_proposal_sort_type),
  )
}
