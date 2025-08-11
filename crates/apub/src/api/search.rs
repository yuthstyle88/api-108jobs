use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::FastJobContext;
use lemmy_api_utils::utils::{check_conflicting_like_filters, check_private_instance};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_search_combined::{
    impls::SearchCombinedQuery, Search, SearchCombinedView, SearchResponse,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::FastJobResult;

pub async fn search(
    data: Query<Search>,
    context: Data<FastJobContext>,
    local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<SearchResponse>> {
    let site_view = SiteView::read_local(&mut context.pool()).await?;
    let local_site = site_view.local_site;

    check_private_instance(&local_user_view, &local_site)?;
    check_conflicting_like_filters(data.liked_only, data.disliked_only)?;

    let cursor_data = if let Some(cursor) = &data.page_cursor {
        Some(SearchCombinedView::from_cursor(cursor, &mut context.pool()).await?)
    } else {
        None
    };

    let pool = &mut context.pool();
    let search_query = SearchCombinedQuery {
        search_term: Some(data.q.clone()),
        community_id: data.community_id,
        creator_id: data.creator_id,
        type_: data.type_,
        sort: data.sort,
        time_range_seconds: data.time_range_seconds,
        listing_type: data.listing_type,
        title_only: data.title_only,
        post_url_only: data.post_url_only,
        liked_only: data.liked_only,
        disliked_only: data.disliked_only,
        cursor_data,
        page_back: data.page_back,
        limit: data.limit,
        self_promotion: None,
    };


    let results = search_query
        .list(pool, &local_user_view, &site_view.site)
        .await?;

    let next_page = results.last().map(PaginationCursorBuilder::to_cursor);
    let prev_page = results.first().map(PaginationCursorBuilder::to_cursor);

    Ok(Json(SearchResponse {
        results,
        next_page,
        prev_page,
    }))
}
