use crate::source::category::CategoryChangeset;
use crate::{
    diesel::{DecoratableTarget, JoinOnDsl, OptionalExtension},
    newtypes::{CategoryId, DbUrl, PersonId},
    source::{
      actor_language::CategoryLanguage,
      category::{
        Category,
        CategoryActions,
        CategoryInsertForm,
        CategoryUpdateForm,
    },
      post::Post,
  },
    traits::{ApubActor, Crud},
    utils::{
    format_actor_url,
    functions::{coalesce, coalesce_2_nullable, lower, random_smallint},
    get_conn,
    uplete,
    DbPool,
  },
};
use chrono::{DateTime, Utc};
use diesel::{
  dsl::{exists, insert_into, not},
  expression::SelectableHelper,
  select,
  update,
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use app_108jobs_db_schema_file::{
  enums::{CategoryVisibility, ListingType},
  schema::{comment, category, category_actions, instance, post},
};
use app_108jobs_utils::error::{FastJobError, FastJobErrorExt, FastJobErrorType, FastJobResult};
use app_108jobs_utils::{
  settings::structs::Settings,
  CACHE_DURATION_LARGEST_CATEGORY,
};
use moka::future::Cache;
use regex::Regex;
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
      ap_id: form.ap_id.clone(),
      local: form.local,
      last_refreshed_at: form.last_refreshed_at,
      icon: form.icon.clone(),
      banner: form.banner.clone(),
      followers_url: form.followers_url.clone(),
      inbox_url: form.inbox_url.clone(),
      moderators_url: form.moderators_url.clone(),
      featured_url: form.featured_url.clone(),
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

#[derive(Debug)]
pub enum CollectionType {
  Moderators,
  Featured,
}

impl Category {
  pub async fn insert_apub(
      pool: &mut DbPool<'_>,
      timestamp: DateTime<Utc>,
      form: &CategoryInsertForm,
  ) -> FastJobResult<Self> {
    let is_new_category = match &form.ap_id {
      Some(id) => Category::read_from_apub_id(pool, id).await?.is_none(),
      None => true,
    };
    let conn = &mut get_conn(pool).await?;

    // Can't do separate insert/update commands because InsertForm/UpdateForm aren't convertible
    let category_ = insert_into(category::table)
     .values(form)
     .on_conflict(category::ap_id)
     .filter_target(coalesce(category::updated_at, category::published_at).lt(timestamp))
     .do_update()
     .set(form)
     .get_result::<Self>(conn)
     .await?;

    // Initialize languages for new category
    if is_new_category {
      CategoryLanguage::update(pool, vec![], category_.id).await?;
    }

    Ok(category_)
  }

  pub async fn list_all_communities(pool: &mut DbPool<'_>) -> FastJobResult<Vec<Category>> {
    let conn = &mut get_conn(pool).await?;

    let communities = category::table
        .order_by(category::path.asc())
        .load::<Category>(conn)
        .await?;

    Ok(communities)
  }

  /// Get the category which has a given moderators or featured url, also return the collection
  /// type
  pub async fn get_by_collection_url(
    pool: &mut DbPool<'_>,
    url: &DbUrl,
  ) -> FastJobResult<(Category, CollectionType)> {
    let conn = &mut get_conn(pool).await?;
    let res = category::table
      .filter(category::moderators_url.eq(url))
      .first(conn)
      .await;

    if let Ok(c) = res {
      Ok((c, CollectionType::Moderators))
    } else {
      let res = category::table
        .filter(category::featured_url.eq(url))
        .first(conn)
        .await;
      if let Ok(c) = res {
        Ok((c, CollectionType::Featured))
      } else {
        Err(FastJobErrorType::NotFound.into())
      }
    }
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

      if let Some(ListingType::Local) = type_ {
        query = query.filter(category::local);
      }

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
    category::removed
      .eq(false)
      .and(category::deleted.eq(false))
  }

  pub fn build_tag_ap_id(&self, tag_name: &str) -> FastJobResult<DbUrl> {
    #[allow(clippy::expect_used)]
    // convert a readable name to an id slug that is appended to the category URL to get a unique
    // tag url (ap_id).
    static VALID_ID_SLUG: LazyLock<Regex> =
      LazyLock::new(|| Regex::new(r"[^a-z0-9_-]+").expect("compile regex"));
    let tag_name_lower = tag_name.to_lowercase();
    let id_slug = VALID_ID_SLUG.replace_all(&tag_name_lower, "-");
    if id_slug.is_empty() {
      Err(FastJobErrorType::InvalidUrl)?
    }
    Ok(Url::parse(&format!("{}/tag/{}", self.ap_id, &id_slug))?.into())
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

    uplete::new(
      category_actions::table.filter(category_actions::category_id.eq(for_category_id)),
    )
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

  /// Check if we should accept activity in remote category. This requires either:
  /// - Local follower of the category
  /// - Local post or comment in the category
  ///
  /// Dont use this check for local communities.
  pub async fn check_accept_activity_in_category(
      pool: &mut DbPool<'_>,
      remote_category_id: CategoryId,
  ) -> FastJobResult<()> {
    let conn = &mut get_conn(pool).await?;
    let follow_action = category_actions::table
      .filter(category_actions::followed_at.is_not_null())
      .filter(category_actions::category_id.eq(remote_category_id));
    let local_post = post::table
      .filter(post::category_id.eq(remote_category_id))
      .filter(post::local);
    let local_comment = comment::table
      .inner_join(post::table)
      .filter(post::category_id.eq(remote_category_id))
      .filter(comment::local);
    select(exists(follow_action).or(exists(local_post).or(exists(local_comment))))
      .get_result::<bool>(conn)
      .await?
      .then_some(())
      .ok_or(FastJobErrorType::CategoryHasNoFollowers.into())
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

impl ApubActor for Category {
  async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: &DbUrl,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    category::table
      .filter(category::ap_id.eq(object_id))
      .first(conn)
      .await
      .optional()
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  async fn read_from_name(
    pool: &mut DbPool<'_>,
    category_name: &str,
    include_deleted: bool,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    let mut q = category::table
      .into_boxed()
      .filter(category::local.eq(true))
      .filter(lower(category::name).eq(category_name.to_lowercase()));
    if !include_deleted {
      q = q.filter(Self::hide_removed_and_deleted())
    }
    q.first(conn)
      .await
      .optional()
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  async fn read_from_name_and_domain(
    pool: &mut DbPool<'_>,
    category_name: &str,
    for_domain: &str,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    category::table
      .inner_join(instance::table)
      .filter(lower(category::name).eq(category_name.to_lowercase()))
      .filter(lower(instance::domain).eq(for_domain.to_lowercase()))
      .select(category::all_columns)
      .first(conn)
      .await
      .optional()
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  fn actor_url(&self, settings: &Settings) -> FastJobResult<Url> {
    let domain = self
      .ap_id
      .inner()
      .domain()
      .ok_or(FastJobErrorType::NotFound)?;

    format_actor_url(&self.name, domain, 'c', settings)
  }

  fn generate_local_actor_url(name: &str, settings: &Settings) -> FastJobResult<DbUrl> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/c/{name}"))?.into())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    source::{
      category::{
          Category,
          CategoryActions,
          CategoryInsertForm,
          CategoryUpdateForm,
      },
      instance::Instance,
      local_user::LocalUser,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    traits::Crud,
    utils::{build_db_pool_for_tests, RANK_DEFAULT},
  };
  use diesel_ltree::Ltree;
  use app_108jobs_utils::error::FastJobResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let bobby_person = PersonInsertForm::test_form(inserted_instance.id, "bobby");
    let inserted_bobby = Person::create(pool, &bobby_person).await?;

    let artemis_person = PersonInsertForm::test_form(inserted_instance.id, "artemis");
    let inserted_artemis = Person::create(pool, &artemis_person).await?;

    let new_category = CategoryInsertForm::new(
      inserted_instance.id,
      "TIL".into(),
      "nada".to_owned(),
    );
    let inserted_category = Category::create(pool, &new_category).await?;

    let expected_category = Category {
      id: inserted_category.id,
      name: "TIL".into(),
      title: "nada".to_owned(),
      sidebar: None,
      description: None,
      self_promotion: false,
      removed: false,
      deleted: false,
      published_at: inserted_category.published_at,
      updated_at: None,
      ap_id: inserted_category.ap_id.clone(),
      local: true,
      last_refreshed_at: inserted_category.published_at,
      icon: None,
      banner: None,
      followers_url: inserted_category.followers_url.clone(),
      inbox_url: inserted_category.inbox_url.clone(),
      moderators_url: None,
      featured_url: None,
      posting_restricted_to_mods: false,
      instance_id: inserted_instance.id,
      visibility: CategoryVisibility::Public,
      random_number: inserted_category.random_number,
      subscribers: 1,
      posts: 0,
      comments: 0,
      users_active_day: 0,
      users_active_week: 0,
      users_active_month: 0,
      users_active_half_year: 0,
      hot_rank: RANK_DEFAULT,
      subscribers_local: 1,
      report_count: 0,
      unresolved_report_count: 0,
      interactions_month: 0,
      local_removed: false,
      path: Ltree("".to_string()),
      active: false,
      is_new: false,
    };



    let moderator_person_ids = vec![inserted_bobby.id, inserted_artemis.id];

    // Make sure bobby is marked as a higher mod than artemis, and vice versa
    let bobby_higher_check = CategoryActions::is_higher_mod_check(
      pool,
      inserted_category.id,
      inserted_bobby.id,
      moderator_person_ids.clone(),
    )
    .await;
    assert!(bobby_higher_check.is_ok());

    // Also check the other is_higher_mod_or_admin function just in case
    let bobby_higher_check_2 = LocalUser::is_higher_mod_or_admin_check(
      pool,
      inserted_category.id,
      inserted_bobby.id,
      moderator_person_ids.clone(),
    )
    .await;
    assert!(bobby_higher_check_2.is_ok());

    // This should throw an error, since artemis was added later
    let artemis_higher_check = CategoryActions::is_higher_mod_check(
      pool,
      inserted_category.id,
      inserted_artemis.id,
      moderator_person_ids,
    )
    .await;
    assert!(artemis_higher_check.is_err());

    let read_category = Category::read(pool, inserted_category.id).await?;

    let update_category_form = CategoryUpdateForm {
      title: Some("nada".to_owned()),
      ..Default::default()
    };
    let updated_category =
      Category::update(pool, inserted_category.id, &update_category_form).await?;

    let num_deleted = Category::delete(pool, inserted_category.id).await?;
    Person::delete(pool, inserted_bobby.id).await?;
    Person::delete(pool, inserted_artemis.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    assert_eq!(expected_category, read_category);
    assert_eq!(expected_category, updated_category);
    assert_eq!(1, num_deleted);

    Ok(())
  }
}
