use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_wallet::api::{ListWalletTopupsQuery, ListWalletTopupsResponse};
use lemmy_db_views_wallet::WalletTopupView;
use lemmy_utils::error::FastJobResult;

pub async fn list_wallet_topups(
  query: Query<ListWalletTopupsQuery>,
  context: Data<FastJobContext>,
  local_user_view: LocalUserView,
) -> FastJobResult<Json<ListWalletTopupsResponse>> {
  let data = query.into_inner();
  let local_user_id = local_user_view.local_user.id;
  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(WalletTopupView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };
  let items =
    WalletTopupView::list(&mut context.pool(), Some(local_user_id), cursor_data, data).await?;
  let next_page = items.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = items.first().map(PaginationCursorBuilder::to_cursor);
  Ok(Json(ListWalletTopupsResponse {
    wallet_topups: items,
    next_page,
    prev_page,
  }))
}
