use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{
  context::FastJobContext,
  utils::{check_private_instance, is_mod_or_admin_opt},
};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_modlog_combined::{
  api::{GetModlog, GetModlogResponse},
  impls::ModlogCombinedQuery,
  ModlogCombinedView,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::FastJobResult;

pub async fn get_mod_log(
  data: Query<GetModlog>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<GetModlogResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;

  check_private_instance(&local_user_view, &local_site)?;

  // Only show the modlog names if:
  // You're an admin or
  // You're fetching the modlog for a single community, and you're a mod
  // (Alternatively !admin/mod)
  let hide_modlog_names = if let Some(community_id) = data.community_id {
    is_mod_or_admin_opt(
      &mut context.pool(),
      local_user_view.as_ref(),
      Some(community_id),
    )
    .await
    .is_err()
  } else {
    !local_user_view
      .as_ref()
      .map(|l| l.local_user.admin)
      .unwrap_or_default()
  };

  // Only allow mod person id filters if its not hidden
  let mod_person_id = if hide_modlog_names {
    None
  } else {
    data.mod_person_id
  };

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(ModlogCombinedView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let modlog = ModlogCombinedQuery {
    type_: data.type_,
    listing_type: data.listing_type,
    community_id: data.community_id,
    mod_person_id,
    other_person_id: data.other_person_id,
    local_user: local_user_view.as_ref().map(|u| &u.local_user),
    post_id: data.post_id,
    comment_id: data.comment_id,
    hide_modlog_names: Some(hide_modlog_names),
    cursor_data,
    page_back: data.page_back,
    limit: data.limit,
  }
  .list(&mut context.pool())
  .await?;

  let next_page = modlog.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = modlog.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(GetModlogResponse {
    modlog,
    next_page,
    prev_page,
  }))
}
