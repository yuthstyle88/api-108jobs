use crate::{
  CategoryView,
  CommentView,
  LocalUserView,
  PersonView,
  PostView,
  SearchCombinedView,
  SearchCombinedViewInternal,
  SearchPostView,
};
use app_108jobs_db_schema::{
  newtypes::{CategoryId, Coin, InstanceId, LanguageId, PaginationCursor, PersonId, PostId},
  source::{
    combined::search::{search_combined_keys as key, SearchCombined},
    site::Site,
  },
  traits::{InternalToCombinedView, PaginationCursorBuilder},
  utils::{
    fuzzy_search,
    get_conn,
    limit_fetch,
    now,
    paginate,
    queries::{
      creator_category_actions_join,
      creator_home_instance_actions_join,
      creator_local_instance_actions_join,
      creator_local_user_admin_join,
      image_details_join,
      my_category_actions_join,
      my_comment_actions_join,
      my_instance_actions_person_join,
      my_local_user_admin_join,
      my_person_actions_join,
      my_post_actions_join,
    },
    seconds_to_pg_interval,
    DbPool,
  },
  SearchSortType::{self, *},
  SearchType,
};
use app_108jobs_db_schema_file::{
  enums::{IntendedUse, JobType, PostKind, TripStatus},
  schema::{category, comment, delivery_details, person, post, ride_session, search_combined},
};
use app_108jobs_db_views_post::logistics::{
  build_logistics_from_maps,
  fetch_logistics_maps_by_ids,
  LogisticsViewer,
};
use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};
use diesel::{
  dsl::{exists, not},
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  PgTextExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::asc_if;

impl SearchCombinedViewInternal {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: Option<PersonId>, local_instance_id: InstanceId) -> _ {
    let item_creator = person::id;

    let item_creator_join = person::table.on(
      search_combined::person_id
        .eq(item_creator.nullable())
        .or(
          search_combined::comment_id
            .is_not_null()
            .and(comment::creator_id.eq(item_creator)),
        )
        .or(
          search_combined::post_id
            .is_not_null()
            .and(post::creator_id.eq(item_creator)),
        )
        .and(not(person::deleted)),
    );

    let comment_join = comment::table.on(
      search_combined::comment_id
        .eq(comment::id.nullable())
        .and(not(comment::removed))
        .and(not(comment::deleted)),
    );

    let post_join = post::table.on(
      search_combined::post_id
        .eq(post::id.nullable())
        .or(comment::post_id.eq(post::id))
        .and(not(post::removed))
        .and(not(post::deleted)),
    );

    let category_join = category::table.on(
      search_combined::category_id
        .eq(category::id.nullable())
        .or(category::id.nullable().eq(post::category_id))
        .and(not(category::removed))
        .and(not(category::local_removed))
        .and(not(category::deleted)),
    );

    let my_category_actions_join: my_category_actions_join = my_category_actions_join(my_person_id);
    let my_post_actions_join: my_post_actions_join = my_post_actions_join(my_person_id);
    let my_comment_actions_join: my_comment_actions_join = my_comment_actions_join(my_person_id);
    let my_local_user_admin_join: my_local_user_admin_join = my_local_user_admin_join(my_person_id);
    let my_instance_actions_person_join: my_instance_actions_person_join =
      my_instance_actions_person_join(my_person_id);
    let my_person_actions_join: my_person_actions_join = my_person_actions_join(my_person_id);
    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);

    search_combined::table
      .left_join(comment_join)
      .left_join(post_join)
      .left_join(item_creator_join)
      .left_join(category_join)
      .left_join(creator_category_actions_join())
      .left_join(my_local_user_admin_join)
      .left_join(creator_local_user_admin_join())
      .left_join(my_category_actions_join)
      .left_join(my_instance_actions_person_join)
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
      .left_join(my_post_actions_join)
      .left_join(my_person_actions_join)
      .left_join(my_comment_actions_join)
      .left_join(image_details_join())
  }
}

impl SearchCombinedView {
  /// Useful in combination with filter_map
  pub fn to_post_view(&self) -> Option<&PostView> {
    if let Self::Post(v) = self {
      Some(&v.post_view)
    } else {
      None
    }
  }
}

impl PaginationCursorBuilder for SearchCombinedView {
  type CursorData = SearchCombined;

  fn to_cursor(&self) -> PaginationCursor {
    let (prefix, id) = match &self {
      SearchCombinedView::Post(v) => ('P', v.post_view.post.id.0),
      SearchCombinedView::Comment(v) => ('C', v.comment.id.0),
      SearchCombinedView::Category(v) => ('O', v.category.id.0),
      SearchCombinedView::Person(v) => ('E', v.person.id.0),
    };
    // Simple: just prefix + hex id (old app_108jobs style)
    PaginationCursor(format!("{}{:x}", prefix, id))
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let s = &cursor.0;
    if s.len() < 2 {
      return Err(FastJobErrorType::CouldntParsePaginationToken.into());
    }
    let prefix = s.chars().next().unwrap();
    let id_hex = &s[1..];
    let id =
      i32::from_str_radix(id_hex, 16).map_err(|_| FastJobErrorType::CouldntParsePaginationToken)?;

    let conn = &mut get_conn(pool).await?;
    let mut query = search_combined::table
      .select(Self::CursorData::as_select())
      .into_boxed();

    query = match prefix {
      'P' => query.filter(search_combined::post_id.eq(id)),
      'C' => query.filter(search_combined::comment_id.eq(id)),
      'O' => query.filter(search_combined::category_id.eq(id)),
      'E' => query.filter(search_combined::person_id.eq(id)),
      _ => return Err(FastJobErrorType::CouldntParsePaginationToken.into()),
    };

    let token = query.first(conn).await?;
    Ok(token)
  }
}

#[derive(Default)]
pub struct SearchCombinedQuery {
  pub search_term: Option<String>,
  pub category_id: Option<CategoryId>,
  pub language_id: Option<LanguageId>,
  pub type_: Option<SearchType>,
  pub creator_id: Option<PersonId>,
  pub sort: Option<SearchSortType>,
  pub time_range_seconds: Option<i32>,
  pub intended_use: Option<IntendedUse>,
  pub job_type: Option<JobType>,
  pub budget_min: Option<Coin>,
  pub budget_max: Option<Coin>,
  pub requires_english: Option<bool>,
  pub post_kind: Option<PostKind>,
  /// Filter by logistics status (Pending, InProgress, Completed, etc.) for Delivery/RideTaxi posts
  pub logistics_status: Option<TripStatus>,
  pub cursor_data: Option<SearchCombined>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

impl SearchCombinedQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: &Option<LocalUserView>,
    site_local: &Site,
  ) -> FastJobResult<Vec<SearchCombinedView>> {
    let my_person_id = user.as_ref().map(|u| u.local_user.person_id);
    let item_creator = person::id;

    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(self.limit)?;

    let mut query = SearchCombinedViewInternal::joins(my_person_id, site_local.instance_id)
      .select(SearchCombinedViewInternal::as_select())
      .limit(limit)
      .into_boxed();

    // Some helpers
    let is_post = search_combined::post_id.is_not_null();
    let is_comment = search_combined::comment_id.is_not_null();
    let is_category = search_combined::category_id.is_not_null();
    let is_person = search_combined::person_id.is_not_null();

    // The search term
    if let Some(search_term) = &self.search_term {
      let searcher = fuzzy_search(search_term);

      let name_or_title_filter = post::name.ilike(searcher.clone());
      let body_or_category_filter = post::body
        .ilike(searcher.clone())
        .or(category::name.ilike(searcher.clone()));
      query = query.filter(name_or_title_filter.or(body_or_category_filter));
    }

    // Category id
    if let Some(category_id) = self.category_id {
      query = query.filter(category::id.eq(category_id));
    }

    if let Some(language_id) = self.language_id {
      query = query.filter(post::language_id.eq(language_id));
    }

    if let Some(req) = self.requires_english {
      query = query.filter(post::is_english_required.eq(req));
    }

    if let Some(min) = self.budget_min {
      query = query.filter(post::budget.ge(min));
    }

    if let Some(max) = self.budget_max {
      query = query.filter(post::budget.le(max));
    }

    if let Some(intended_use) = self.intended_use {
      query = query.filter(post::intended_use.eq(intended_use));
    }

    if let Some(job_type) = self.job_type {
      query = query.filter(post::job_type.eq(job_type));
    }

    // Creator id
    if let Some(creator_id) = self.creator_id {
      query = query.filter(item_creator.eq(creator_id));
    }

    // Liked / disliked filter
    if let Some(my_id) = my_person_id {
      // TODO: implement like/dislike filtering based on user preferences.
      // The previous closure here was unused and triggered a compiler warning.
      let _ = item_creator.ne(my_id);
    };

    query = match self.type_.unwrap_or_default() {
      SearchType::All => query,
      SearchType::Posts => query.filter(is_post),
      SearchType::Comments => query.filter(is_comment),
      SearchType::Categories => query.filter(is_category),
      SearchType::Users => query.filter(is_person),
    };

    // Filter by post_kind (Delivery, RideTaxi, Normal), defaults to Normal
    query = query.filter(post::post_kind.eq(self.post_kind.unwrap_or(PostKind::Normal)));

    // Filter by logistics status (for Delivery and RideTaxi posts)
    // Uses EXISTS subqueries to avoid joins
    if let Some(status) = self.logistics_status {
      match self.post_kind.unwrap_or(PostKind::Normal) {
        PostKind::Delivery => {
          // Filter delivery posts by delivery_details status
          query = query.filter(exists(
            delivery_details::table
              .filter(delivery_details::post_id.eq(post::id))
              .filter(delivery_details::status.eq(status)),
          ));
        }
        PostKind::RideTaxi => {
          // Filter ride taxi posts by ride_session status
          query = query.filter(exists(
            ride_session::table
              .filter(ride_session::post_id.eq(post::id))
              .filter(ride_session::status.eq(status)),
          ));
        }
        PostKind::Normal => {
          // No logistics for Normal posts - don't apply filter
        }
      }
    }

    // Filter by the time range
    if let Some(time_range_seconds) = self.time_range_seconds {
      query = query.filter(
        search_combined::published_at.gt(now() - seconds_to_pg_interval(time_range_seconds)),
      );
    }

    // Only sort by asc if old
    let sort = self.sort.unwrap_or_default();
    let sort_direction = asc_if(sort == Old);

    let mut paginated_query = paginate(
      query,
      sort_direction,
      self.cursor_data,
      None,
      self.page_back,
    );

    paginated_query = match sort {
      New | Old => paginated_query.then_order_by(key::published_at),
      Top => paginated_query.then_order_by(key::score),
    }
    // finally use unique id as tie breaker
    .then_order_by(key::id);

    let res = paginated_query
      .load::<SearchCombinedViewInternal>(conn)
      .await?;

    // Map the query results to the enum
    let out: Vec<SearchCombinedView> = res
      .into_iter()
      .filter_map(InternalToCombinedView::map_to_enum)
      .collect();

    Ok(out)
  }
}

/// Batch load logistics for all post results in a search response.
/// Optimized: Skips DB queries entirely if all posts are Normal kind.
/// Reuses shared logistics functions from db_views_post.
pub async fn load_logistics_for_results(
  results: &mut Vec<SearchCombinedView>,
  pool: &mut DbPool<'_>,
  viewer: LogisticsViewer,
  is_admin: bool,
) -> FastJobResult<()> {
  // Early return if no results
  if results.is_empty() {
    return Ok(());
  }

  // Check if any posts need logistics (before getting DB connection)
  let has_delivery = results.iter().any(|r| {
    matches!(r, SearchCombinedView::Post(spv) if spv.post_view.post.post_kind == PostKind::Delivery)
  });
  let has_ride = results.iter().any(|r| {
    matches!(r, SearchCombinedView::Post(spv) if spv.post_view.post.post_kind == PostKind::RideTaxi)
  });

  // Early return if no delivery or ride posts - skip DB entirely
  if !has_delivery && !has_ride {
    return Ok(());
  }

  let conn = &mut get_conn(pool).await?;

  // Collect post IDs by kind in a single pass
  let (delivery_ids, ride_ids): (Vec<PostId>, Vec<PostId>) = results
    .iter()
    .filter_map(|r| match r {
      SearchCombinedView::Post(spv) => Some(spv),
      _ => None,
    })
    .fold((Vec::new(), Vec::new()), |(mut del, mut ride), spv| {
      match spv.post_view.post.post_kind {
        PostKind::Delivery => del.push(spv.post_view.post.id),
        PostKind::RideTaxi => ride.push(spv.post_view.post.id),
        PostKind::Normal => {}
      }
      (del, ride)
    });

  // Fetch maps using shared function
  let maps = fetch_logistics_maps_by_ids(conn, &delivery_ids, &ride_ids).await?;

  // Assign logistics to each post result using shared function
  for result in results.iter_mut() {
    if let SearchCombinedView::Post(search_post_view) = result {
      search_post_view.logistics =
        build_logistics_from_maps(&search_post_view.post_view, &maps, viewer, is_admin);
    }
  }

  Ok(())
}

impl InternalToCombinedView for SearchCombinedViewInternal {
  type CombinedView = SearchCombinedView;

  fn map_to_enum(self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self;

    if let (Some(comment), Some(creator), Some(post)) =
      (v.comment, v.item_creator.clone(), v.post.clone())
    {
      Some(SearchCombinedView::Comment(CommentView {
        comment,
        post,
        category: v.category,
        creator,
        category_actions: v.category_actions,
        instance_actions: v.instance_actions,
        person_actions: v.person_actions,
        comment_actions: v.comment_actions,
        creator_is_admin: v.item_creator_is_admin,
        post_tags: v.post_tags,
        can_mod: v.can_mod,
        creator_banned: v.creator_banned,
        creator_is_moderator: v.creator_is_moderator,
        creator_banned_from_category: v.creator_banned_from_category,
      }))
    } else if let (Some(post), Some(creator)) = (v.post, v.item_creator.clone()) {
      let post_view = PostView {
        post,
        category: v.category,
        creator,
        creator_is_admin: v.item_creator_is_admin,
        image_details: v.image_details,
        category_actions: v.category_actions,
        instance_actions: v.instance_actions,
        person_actions: v.person_actions,
        post_actions: v.post_actions,
        tags: v.post_tags,
        can_mod: v.can_mod,
        creator_banned: v.creator_banned,
        creator_is_moderator: v.creator_is_moderator,
        creator_banned_from_category: v.creator_banned_from_category,
      };
      Some(SearchCombinedView::Post(SearchPostView {
        post_view,
        logistics: None, // Logistics loaded separately via load_logistics_for_results
      }))
    } else if let Some(category) = v.category {
      Some(SearchCombinedView::Category(CategoryView {
        category,
        category_actions: v.category_actions,
        instance_actions: v.instance_actions,
        can_mod: v.can_mod,
        post_tags: v.category_post_tags,
      }))
    } else if let Some(person) = v.item_creator {
      Some(SearchCombinedView::Person(PersonView {
        person,
        is_admin: v.item_creator_is_admin,
        person_actions: v.person_actions,
        creator_banned: v.creator_banned,
      }))
    } else {
      None
    }
  }
}
