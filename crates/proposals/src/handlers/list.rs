use actix_web::web::{Data, Json, Query};
use app_108jobs_api_utils::{
  context::FastJobContext,
  listing_defaults::{listing_type_with_default, proposal_sort_type_with_default},
  utils::check_private_instance,
};
use app_108jobs_core::error::{FastJobErrorType, FastJobResult};
use app_108jobs_db::{
  newtypes::PaginationCursor,
  source::{category::Category, proposal::Proposal},
  traits::{Crud, PaginationCursorBuilder},
};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_proposal::{
  api::{GetComments, GetCommentsResponse, GetCommentsSlimResponse},
  impls::ProposalQuery,
  ProposalView,
};

struct CommentsCommonOutput {
  proposals: Vec<ProposalView>,
  next_page: Option<PaginationCursor>,
  prev_page: Option<PaginationCursor>,
}

/// A common fetcher for both the ProposalView, and ProposalSlimView.
async fn list_comments_common(
  data: Query<GetComments>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<CommentsCommonOutput> {
  let site_view = context.site_config().get().await?.site_view;
  check_private_instance(&local_user_view, &site_view.local_site)?;

  let category_id = if let Some(name) = &data.category_name {
    Some(
      Category::read_from_name(&mut context.pool(), name, true)
        .await?
        .ok_or(FastJobErrorType::NotFound)?
        .id,
    )
  } else {
    data.category_id
  };
  let local_user_ref = local_user_view.as_ref().map(|u| &u.local_user);
  let sort = Some(proposal_sort_type_with_default(
    data.sort,
    local_user_ref,
    &site_view.local_site,
  ));
  let time_range_seconds = data.time_range_seconds;
  let max_depth = data.max_depth;
  let limit = data.limit;
  let parent_id = data.parent_id;

  let listing_type = Some(listing_type_with_default(
    data.type_,
    local_user_view.as_ref().map(|u| &u.local_user),
    &site_view.local_site,
    category_id,
  ));

  // If a parent_id is given, fetch the comment to get the path
  let parent_path_ = if let Some(parent_id) = parent_id {
    Some(Proposal::read(&mut context.pool(), parent_id).await?.path)
  } else {
    None
  };

  let parent_path = parent_path_.clone();
  let post_id = data.post_id;
  let local_user = local_user_view.as_ref().map(|l| &l.local_user);

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(ProposalView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };
  let page_back = data.page_back;

  let comments = ProposalQuery {
    listing_type,
    sort,
    time_range_seconds,
    max_depth,
    category_id,
    post_id,
    parent_path,
    local_user,
    cursor_data,
    page_back,
    limit,
  }
  .list(&site_view.site, &mut context.pool())
  .await?;

  let next_page = comments.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = comments.first().map(PaginationCursorBuilder::to_cursor);

  Ok(CommentsCommonOutput {
    proposals: comments,
    next_page,
    prev_page,
  })
}

pub async fn list_comments(
  data: Query<GetComments>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<GetCommentsResponse>> {
  let common = list_comments_common(data, context, local_user_view).await?;

  Ok(Json(GetCommentsResponse {
    proposals: common.proposals,
    next_page: common.next_page,
    prev_page: common.prev_page,
  }))
}

pub async fn list_comments_slim(
  data: Query<GetComments>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<GetCommentsSlimResponse>> {
  let common = list_comments_common(data, context, local_user_view).await?;

  let comments = common
    .proposals
    .into_iter()
    .map(ProposalView::map_to_slim)
    .collect();

  Ok(Json(GetCommentsSlimResponse {
    proposals: comments,
    next_page: common.next_page,
    prev_page: common.prev_page,
  }))
}
