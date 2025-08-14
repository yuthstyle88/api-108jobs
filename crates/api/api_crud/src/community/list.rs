use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::build_response::build_community_tree;
use lemmy_api_utils::{context::FastJobContext, utils::check_private_instance};
use lemmy_db_schema::newtypes::PaginationCursor;
use lemmy_db_schema::source::community::Community;
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_schema::CommunitySortType;
use lemmy_db_schema_file::enums::ListingType;
use lemmy_db_views_community::api::ListCommunitiesTreeResponse;
use lemmy_db_views_community::{
  api::{ListCommunities, ListCommunitiesResponse},
  impls::CommunityQuery,
  CommunityView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::FastJobResult;
use moka::future::Cache;
use std::hash::{Hash, Hasher};
use std::{sync::LazyLock, time::Duration};

// Define a cache key type based on query parameters
#[derive(Clone, Debug, Eq)]
struct CommunitiesListCacheKey {
  type_: Option<ListingType>,
  sort: Option<CommunitySortType>,
  time_range_seconds: Option<i32>,
  max_depth: Option<i32>,
  limit: Option<i64>,
  self_promotion: Option<bool>,
  page_cursor: Option<PaginationCursor>,
  page_back: Option<bool>,
  is_authenticated: bool,
}

impl PartialEq for CommunitiesListCacheKey {
  fn eq(&self, other: &Self) -> bool {
    self.type_ == other.type_
      && self.sort == other.sort
      && self.time_range_seconds == other.time_range_seconds
      && self.max_depth == other.max_depth
      && self.limit == other.limit
      && self.self_promotion == other.self_promotion
      && self.page_cursor == other.page_cursor
      && self.page_back == other.page_back
      && self.is_authenticated == other.is_authenticated
  }
}

impl Hash for CommunitiesListCacheKey {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.type_.hash(state);
    self.sort.hash(state);
    self.time_range_seconds.hash(state);
    self.max_depth.hash(state);
    self.limit.hash(state);
    self.self_promotion.hash(state);
    self.page_cursor.hash(state);
    self.page_back.hash(state);
    self.is_authenticated.hash(state);
  }
}

// Create a static cache instance with a 5-minute expiration time
static COMMUNITIES_CACHE: LazyLock<Cache<CommunitiesListCacheKey, ListCommunitiesResponse>> =
  LazyLock::new(|| {
    Cache::builder()
      // Set a reasonable maximum size to prevent memory issues
      .max_capacity(1000)
      // Set a 5-minute time-to-live for cache entries
      .time_to_live(Duration::from_secs(300))
      // Set a 10-minute time-to-idle for cache entries
      .time_to_idle(Duration::from_secs(600))
      .build()
  });

pub async fn list_communities(
  data: Query<ListCommunities>,
  context: Data<FastJobContext>,
  local_user_view: Option<LocalUserView>,
) -> FastJobResult<Json<ListCommunitiesResponse>> {
  // Check private instance first to avoid unnecessary processing
  let site_view = context.site_config().get().await?.site_view;
  check_private_instance(&local_user_view, &site_view.local_site)?;

  // Create a cache key based on the query parameters
  let cache_key = CommunitiesListCacheKey {
    type_: data.type_,
    sort: data.sort,
    time_range_seconds: data.time_range_seconds,
    max_depth: data.max_depth,
    limit: data.limit,
    self_promotion: data.self_promotion,
    page_cursor: data.page_cursor.clone(),
    page_back: data.page_back,
    is_authenticated: local_user_view.is_some(),
  };

  // Only use cache for standard listings (not for authenticated users with custom views)
  // and when not using pagination (which is likely to be unique per user)
  let use_cache = !cache_key.is_authenticated && cache_key.page_cursor.is_none();

  // Try to get the response from the cache if caching is enabled for this request
  if use_cache {
    if let Some(cached_response) = COMMUNITIES_CACHE.get(&cache_key).await {
      return Ok(Json(cached_response));
    }
  }

  // If not in cache or caching disabled, fetch from database
  let local_user = local_user_view.map(|l| l.local_user);

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(CommunityView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  // Show self_promotion content if param is true, or if content_warning exists
  let self_promotion = data
    .self_promotion
    .unwrap_or(site_view.site.content_warning.is_some());

  let communities = CommunityQuery {
    listing_type: data.type_,
    self_promotion: Some(self_promotion),
    sort: data.sort,
    time_range_seconds: data.time_range_seconds,
    max_depth: data.max_depth,
    local_user: local_user.as_ref(),
    cursor_data,
    page_back: data.page_back,
    limit: data.limit,
    ..Default::default()
  }
  .list(&site_view.site, &mut context.pool())
  .await?;

  let next_page = communities.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = communities.first().map(PaginationCursorBuilder::to_cursor);

  // Create the response
  let response = ListCommunitiesResponse {
    communities,
    next_page,
    prev_page,
  };

  // Store in cache if caching is enabled for this request
  if use_cache {
    COMMUNITIES_CACHE.insert(cache_key, response.clone()).await;
  }

  // Return the response
  Ok(Json(response))
}

pub async fn list_communities_ltree(
  context: Data<FastJobContext>,
) -> FastJobResult<Json<ListCommunitiesTreeResponse>> {
  let flat_list = Community::list_all_communities(&mut context.pool()).await?;

  build_community_tree(flat_list)
}
