use crate::{
  api::{CreateCategory, CreateCategoryRequest},
  CategoryView,
};
use app_108jobs_core::{
  error::{FastJobError, FastJobErrorExt, FastJobErrorType, FastJobResult},
  utils::validation::get_required_trimmed,
};
use app_108jobs_db::{
  enums::ListingType,
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CategoryId, PaginationCursor, PersonId},
  schema::{category, category_actions},
  source::{
    category::{category_keys as key, Category},
    local_user::LocalUser,
    site::Site,
  },
  traits::{Crud, PaginationCursorBuilder},
  utils::{
    get_conn,
    limit_fetch,
    now,
    paginate,
    queries::{
      filter_is_subscribed,
      filter_not_unlisted_or_is_subscribed,
      my_category_actions_join,
      my_instance_actions_category_join,
      my_local_user_admin_join,
    },
    seconds_to_pg_interval,
    DbPool,
    LowerKey,
  },
  CategorySortType,
};
use diesel::{debug_query, pg::Pg, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use diesel_ltree::nlevel;
use i_love_jesus::asc_if;

impl CategoryView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(person_id: Option<PersonId>) -> _ {
    let category_actions_join: my_category_actions_join = my_category_actions_join(person_id);
    let instance_actions_category_join: my_instance_actions_category_join =
      my_instance_actions_category_join(person_id);
    let my_local_user_admin_join: my_local_user_admin_join = my_local_user_admin_join(person_id);

    category::table
      .left_join(category_actions_join)
      .left_join(instance_actions_category_join)
      .left_join(my_local_user_admin_join)
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    category_id: CategoryId,
    my_local_user: Option<&'_ LocalUser>,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    let mut query = Self::joins(my_local_user.person_id())
      .filter(category::id.eq(category_id))
      .select(Self::as_select())
      .into_boxed();

    query = my_local_user.visible_communities_only(query);

    query
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}

impl PaginationCursorBuilder for CategoryView {
  type CursorData = Category;
  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_single('C', self.category.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let id = cursor.first_id()?;
    Category::read(pool, CategoryId(id)).await
  }
}

#[derive(Default)]
pub struct CategoryQuery<'a> {
  pub listing_type: Option<ListingType>,
  pub sort: Option<CategorySortType>,
  pub time_range_seconds: Option<i32>,
  pub max_depth: Option<i32>,
  pub local_user: Option<&'a LocalUser>,
  pub self_promotion: Option<bool>,
  pub cursor_data: Option<Category>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

impl CategoryQuery<'_> {
  pub async fn list(self, site: &Site, pool: &mut DbPool<'_>) -> FastJobResult<Vec<CategoryView>> {
    use app_108jobs_db::CategorySortType::*;
    let conn = &mut get_conn(pool).await?;
    let o = self;
    let limit = limit_fetch(o.limit)?;

    let mut query = CategoryView::joins(o.local_user.person_id())
      .select(CategoryView::as_select())
      .limit(limit)
      .into_boxed();

    // Hide deleted and removed for non-admins
    let is_admin = o.local_user.map(|l| l.admin).unwrap_or_default();
    if !is_admin {
      query = query
        .filter(Category::hide_removed_and_deleted())
        .filter(filter_not_unlisted_or_is_subscribed());
    }

    if let Some(listing_type) = o.listing_type {
      query = match listing_type {
        ListingType::All => query.filter(filter_not_unlisted_or_is_subscribed()),
        ListingType::Subscribed => query.filter(filter_is_subscribed()),
        // Federation removed: all categories are local, no ::local filter needed
        ListingType::Local => query.filter(filter_not_unlisted_or_is_subscribed()),
        ListingType::ModeratorView => {
          query.filter(category_actions::became_moderator_at.is_not_null())
        }
      };
    }

    // Don't show blocked communities and communities on blocked instances. self_promotion
    // communities are also hidden (based on profile setting)
    if !(o.local_user.self_promotion(site) || o.self_promotion.unwrap_or_default()) {
      query = query.filter(category::self_promotion.eq(false));
    }

    if let Some(depth) = o.max_depth {
      query = query.filter(nlevel(category::path).le(depth));
    }

    query = o.local_user.visible_communities_only(query);

    // Filter by the time range
    if let Some(time_range_seconds) = o.time_range_seconds {
      query =
        query.filter(category::published_at.gt(now() - seconds_to_pg_interval(time_range_seconds)));
    }

    // Only sort by ascending for Old or NameAsc sorts.
    let sort = o.sort.unwrap_or_default();
    let sort_direction = asc_if(sort == Old || sort == NameAsc);
    println!("SQL before pagination: {}", debug_query::<Pg, _>(&query));
    let mut pq = paginate(query, sort_direction, o.cursor_data, None, o.page_back);

    pq = match sort {
      Hot => pq.then_order_by(key::hot_rank),
      Comments => pq.then_order_by(key::comments),
      Posts => pq.then_order_by(key::posts),
      New => pq.then_order_by(key::published_at),
      Old => pq.then_order_by(key::published_at),
      Subscribers => pq.then_order_by(key::subscribers),
      SubscribersLocal => pq.then_order_by(key::subscribers_local),
      ActiveSixMonths => pq.then_order_by(key::users_active_half_year),
      ActiveMonthly => pq.then_order_by(key::users_active_month),
      ActiveWeekly => pq.then_order_by(key::users_active_week),
      ActiveDaily => pq.then_order_by(key::users_active_day),
      NameAsc => pq.then_order_by(LowerKey(key::name)),
      NameDesc => pq.then_order_by(LowerKey(key::name)),
    };

    // finally use unique id as tie breaker
    pq = pq.then_order_by(key::id);

    pq.load::<CategoryView>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}

impl TryFrom<CreateCategoryRequest> for CreateCategory {
  type Error = FastJobError;

  fn try_from(mut value: CreateCategoryRequest) -> Result<Self, Self::Error> {
    let name = get_required_trimmed(&value.name, FastJobErrorType::EmptyTitle)?;

    let title = value.title.take().unwrap_or_default();

    Ok(CreateCategory {
      name,
      title,
      sidebar: None,
      description: value.description.take(),
      icon: value.icon.take(),
      banner: value.banner.take(),
      self_promotion: value.self_promotion.take(),
      posting_restricted_to_mods: None,
      discussion_languages: None,
      visibility: None,
      is_new: value.is_new.take(),
      parent_id: value.parent_id.take(),
    })
  }
}
