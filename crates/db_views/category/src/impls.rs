use crate::{
  api::{CreateCategory, CreateCategoryRequest},
  CategoryView,
};
use app_108jobs_db_schema::{
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CategoryId, PaginationCursor, PersonId},
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
use app_108jobs_db_schema_file::{
  enums::ListingType,
  schema::{category, category_actions},
};
use app_108jobs_utils::{
  error::{FastJobError, FastJobErrorExt, FastJobErrorType, FastJobResult},
  utils::validation::get_required_trimmed,
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
    use app_108jobs_db_schema::CategorySortType::*;
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
        ListingType::Local => query
          .filter(category::local.eq(true))
          .filter(filter_not_unlisted_or_is_subscribed()),
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

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
  use crate::{impls::CategoryQuery, CategoryView};
  use app_108jobs_db_schema::{
    source::{
      category::{Category, CategoryInsertForm, CategoryUpdateForm},
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      site::Site,
    },
    traits::Crud,
    utils::{build_db_pool_for_tests, DbPool},
    CategorySortType,
  };
  use app_108jobs_db_schema_file::enums::{CategoryFollowerState, CategoryVisibility};
  use app_108jobs_utils::error::{FastJobErrorType, FastJobResult};
  use serial_test::serial;
  use std::collections::HashSet;
  use url::Url;

  struct Data {
    instance: Instance,
    local_user: LocalUser,
    communities: [Category; 3],
    site: Site,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> FastJobResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let person_name = "tegan".to_string();

    let (new_person, _) =
      PersonInsertForm::test_form_with_wallet(pool, instance.id, &person_name).await?;

    let inserted_person = Person::create(pool, &new_person).await?;

    let local_user_form = LocalUserInsertForm::test_form(inserted_person.id);
    let local_user = LocalUser::create(pool, &local_user_form, vec![]).await?;

    let communities = [
      Category::create(
        pool,
        &CategoryInsertForm::new(
          instance.id,
          "test_category_1".to_string(),
          "nada1".to_owned(),
        ),
      )
      .await?,
      Category::create(
        pool,
        &CategoryInsertForm::new(
          instance.id,
          "test_category_2".to_string(),
          "nada2".to_owned(),
        ),
      )
      .await?,
      Category::create(
        pool,
        &CategoryInsertForm::new(
          instance.id,
          "test_category_3".to_string(),
          "nada3".to_owned(),
        ),
      )
      .await?,
    ];

    let url = Url::parse("http://example.com")?;
    let site = Site {
      id: Default::default(),
      name: String::new(),
      sidebar: None,
      published_at: Default::default(),
      updated_at: None,
      icon: None,
      banner: None,
      description: None,
      ap_id: Url::parse("http://example1.com")?.into(),
      last_refreshed_at: Default::default(),
      inbox_url: url.into(),
      private_key: None,
      public_key: "".to_string(),
      instance_id: Default::default(),
      content_warning: None,
    };

    Ok(Data {
      instance,
      local_user,
      communities,
      site,
    })
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> FastJobResult<()> {
    for Category { id, .. } in data.communities {
      Category::delete(pool, id).await?;
    }
    Person::delete(pool, data.local_user.person_id).await?;
    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

}
