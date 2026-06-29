use crate::{
  diesel::{JoinOnDsl, OptionalExtension},
  enums::{CategoryVisibility, ListingType},
  newtypes::{CategoryId, PersonId},
  schema::{category, category_actions, post},
  source::{
    actor_language::CategoryLanguage,
    category::{
      Category, CategoryActions, CategoryChangeset, CategoryInsertForm, CategoryUpdateForm,
    },
    post::Post,
  },
  traits::Crud,
  utils::{
    functions::{coalesce_2_nullable, lower, random_smallint},
    get_conn, uplete, DbPool,
  },
};
use app_108jobs_core::{
  error::{FastJobError, FastJobErrorExt, FastJobErrorType, FastJobResult},
  settings::structs::Settings,
  CACHE_DURATION_LARGEST_CATEGORY,
};
use diesel::{
  dsl::{insert_into, not},
  expression::SelectableHelper,
  select, update, BoolExpressionMethods, ExpressionMethods, NullableExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use moka::future::Cache;
use std::sync::{Arc, LazyLock};
use url::Url;

impl Crud for Category {
  type InsertForm = CategoryInsertForm;
  type UpdateForm = CategoryUpdateForm;
  type IdType = CategoryId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    let category_ = insert_into(category::table)
      .values(form)
      .get_result::<Self>(conn)
      .await?;

    // Initialize languages for new category
    CategoryLanguage::update(pool, vec![], category_.id).await?;

    Ok(category_)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    category_id: CategoryId,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    // Create a changeset from the form
    let changeset = CategoryChangeset {
      title: form.title.clone(),
      sidebar: form.sidebar.clone(),
      removed: form.removed,
      published_at: form.published_at,
      updated_at: form.updated_at,
      deleted: form.deleted,
      self_promotion: form.self_promotion,
      last_refreshed_at: form.last_refreshed_at,
      icon: form.icon.clone(),
      banner: form.banner.clone(),
      posting_restricted_to_mods: form.posting_restricted_to_mods,
      visibility: form.visibility,
      description: form.description.clone(),
      local_removed: form.local_removed,
    };

    // Execute the update statement with the explicit changeset
    update(category::table.find(category_id))
      .set(changeset)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateCategory)
  }
}

impl Category {
  pub async fn read_from_name(
    pool: &mut DbPool<'_>,
    category_name: &str,
    include_deleted: bool,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    let mut q = category::table
      .into_boxed()
      .filter(lower(category::name).eq(category_name.to_lowercase()));
    if !include_deleted {
      q = q.filter(Self::hide_removed_and_deleted())
    }
    q.first(conn)
      .await
      .optional()
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub fn actor_url(&self, settings: &Settings) -> FastJobResult<Url> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/c/{}", self.name))?)
  }

  pub async fn list_all_communities(pool: &mut DbPool<'_>) -> FastJobResult<Vec<Category>> {
    let conn = &mut get_conn(pool).await?;

    let communities = category::table
      .order_by(category::path.asc())
      .load::<Category>(conn)
      .await?;

    Ok(communities)
  }

  pub async fn set_featured_posts(
    category_id: CategoryId,
    posts: Vec<Post>,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<()> {
    let conn = &mut get_conn(pool).await?;
    for p in &posts {
      debug_assert!(p.category_id == Some(category_id));
    }
    // Mark the given posts as featured and all other posts as not featured.
    let post_ids = posts.iter().map(|p| p.id);
    update(post::table)
      .filter(post::category_id.eq(category_id))
      // This filter is just for performance
      .filter(post::featured_category.or(post::id.eq_any(post_ids.clone())))
      .set(post::featured_category.eq(post::id.eq_any(post_ids)))
      .execute(conn)
      .await?;
    Ok(())
  }

  pub async fn get_random_category_id(
    pool: &mut DbPool<'_>,
    type_: &Option<ListingType>,
    self_promotion: Option<bool>,
  ) -> FastJobResult<CategoryId> {
    let conn = &mut get_conn(pool).await?;

    // This is based on the random page selection algorithm in MediaWiki. It assigns a random number
    // X to each item. To pick a random one, it generates a random number Y and gets the item with
    // the lowest X value where X >= Y.
    //
    // https://phabricator.wikimedia.org/source/mediawiki/browse/master/includes/specials/SpecialRandomPage.php;763c5f084101676ab1bc52862e1ffbd24585a365
    //
    // The difference is we also regenerate the item's assigned number when the item is picked.
    // Without this, items would have permanent variations in the probability of being picked.
    // Additionally, in each group of multiple items that are assigned the same random number (a
    // more likely occurence with `smallint`), there would be only one item that ever gets
    // picked.

    let try_pick = || {
      let mut query = category::table
        .filter(not(
          category::deleted
            .or(category::removed)
            .or(category::visibility.eq(CategoryVisibility::Private)),
        ))
        .order(category::random_number.asc())
        .select(category::id)
        .into_boxed();

      // Federation removed: all categories are local, so ListingType::Local has no distinct effect.
      let _ = type_;

      if !self_promotion.unwrap_or(false) {
        query = query.filter(not(category::self_promotion));
      }

      query
    };

    diesel::update(category::table)
      .filter(
        category::id.nullable().eq(coalesce_2_nullable(
          try_pick()
            .filter(category::random_number.nullable().ge(
              // Without `select` and `single_value`, this would call `random_smallint` separately
              // for each row
              select(random_smallint()).single_value(),
            ))
            .single_value(),
          // Wrap to the beginning if the generated number is higher than all
          // `category::random_number` values, just like in the MediaWiki algorithm
          try_pick().single_value(),
        )),
      )
      .set(category::random_number.eq(random_smallint()))
      .returning(category::id)
      .get_result::<CategoryId>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  #[diesel::dsl::auto_type(no_type_alias)]
  pub fn hide_removed_and_deleted() -> _ {
    category::removed.eq(false).and(category::deleted.eq(false))
  }
}

impl CategoryActions {
  pub async fn read(
    pool: &mut DbPool<'_>,
    category_id: CategoryId,
    person_id: PersonId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    category_actions::table
      .find((person_id, category_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn delete_mods_for_category(
    pool: &mut DbPool<'_>,
    for_category_id: CategoryId,
  ) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;

    uplete::new(category_actions::table.filter(category_actions::category_id.eq(for_category_id)))
      .set_null(category_actions::became_moderator_at)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn leave_mod_team_for_all_communities(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
  ) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(category_actions::table.filter(category_actions::person_id.eq(for_person_id)))
      .set_null(category_actions::became_moderator_at)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn get_person_moderated_communities(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
  ) -> FastJobResult<Vec<CategoryId>> {
    let conn = &mut get_conn(pool).await?;
    category_actions::table
      .filter(category_actions::became_moderator_at.is_not_null())
      .filter(category_actions::person_id.eq(for_person_id))
      .select(category_actions::category_id)
      .load::<CategoryId>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  /// Checks to make sure the acting moderator was added earlier than the target moderator
  pub async fn is_higher_mod_check(
    pool: &mut DbPool<'_>,
    for_category_id: CategoryId,
    mod_person_id: PersonId,
    target_person_ids: Vec<PersonId>,
  ) -> FastJobResult<()> {
    let conn = &mut get_conn(pool).await?;

    // Build the list of persons
    let mut persons = target_person_ids;
    persons.push(mod_person_id);
    persons.dedup();

    let res = category_actions::table
      .filter(category_actions::became_moderator_at.is_not_null())
      .filter(category_actions::category_id.eq(for_category_id))
      .filter(category_actions::person_id.eq_any(persons))
      .order_by(category_actions::became_moderator_at)
      .select(category_actions::person_id)
      // This does a limit 1 select first
      .first::<PersonId>(conn)
      .await?;

    // If the first result sorted by published is the acting mod
    if res == mod_person_id {
      Ok(())
    } else {
      Err(FastJobErrorType::NotHigherMod)?
    }
  }

  pub async fn fetch_largest_subscribed_category(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> FastJobResult<Option<CategoryId>> {
    static CACHE: LazyLock<Cache<PersonId, Option<CategoryId>>> = LazyLock::new(|| {
      Cache::builder()
        .max_capacity(1000)
        .time_to_live(CACHE_DURATION_LARGEST_CATEGORY)
        .build()
    });
    CACHE
      .try_get_with(person_id, async move {
        let conn = &mut get_conn(pool).await?;
        category_actions::table
          .filter(category_actions::followed_at.is_not_null())
          .filter(category_actions::person_id.eq(person_id))
          .inner_join(category::table.on(category::id.eq(category_actions::category_id)))
          .order_by(category::users_active_month.desc())
          .select(category::id)
          .first::<CategoryId>(conn)
          .await
          .optional()
          .with_fastjob_type(FastJobErrorType::NotFound)
      })
      .await
      .map_err(|_e: Arc<FastJobError>| FastJobErrorType::NotFound.into())
  }
}
