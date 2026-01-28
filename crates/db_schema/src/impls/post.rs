use crate::{
    newtypes::{CategoryId, InstanceId, PaginationCursor, PersonId, PostId},
    source::post::{
    Post,
    PostActions,
    PostHideForm,
    PostInsertForm,
    PostLikeForm,
    PostReadCommentsForm,
    PostReadForm,
    PostSavedForm,
    PostUpdateForm,
  },
    traits::{Crud, Hideable, Likeable, ReadComments, Readable, Saveable},
    utils::{
    functions::{coalesce, hot_rank, scaled_rank},
    get_conn,
    now,
    uplete,
    validate_like,
    DbPool,
    DELETED_REPLACEMENT_TEXT,
    FETCH_LIMIT_MAX,
  },
};
use chrono::{DateTime, Utc};
use diesel::{debug_query, dsl::{count, insert_into, not, update}, expression::SelectableHelper, BoolExpressionMethods, DecoratableTarget, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, OptionalExtension, QueryDsl};
use diesel::dsl::{exists, select};
use diesel::pg::Pg;
use diesel_async::RunQueryDsl;
use tracing::log::debug;
use app_108jobs_db_schema_file::{
  enums::PostNotifications,
  schema::{category, person, post, post_actions},
};
use app_108jobs_utils::{
  error::{FastJobErrorExt, FastJobErrorExt2, FastJobErrorType, FastJobResult},
  settings::structs::Settings,
};
use url::Url;
use crate::newtypes::DbUrl;
use crate::utils::functions::lower;

impl Crud for Post {
  type InsertForm = PostInsertForm;
  type UpdateForm = PostUpdateForm;
  type IdType = PostId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    let query = insert_into(post::table).values(form);

    let sql_preview = debug_query::<Pg, _>(&query).to_string();
    debug!("SQL preview: {sql_preview}");
    let res = query
     .get_result::<Self>(conn)
     .await
     .with_fastjob_type(FastJobErrorType::CouldntCreatePost);
    res
  }

  async fn update(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    new_post: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(post::table.find(post_id))
      .set(new_post)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdatePost)
  }
}

impl Post {
  pub async fn read_xx(pool: &mut DbPool<'_>, id: PostId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    post::table
      .find(id)
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn insert_apub(
    pool: &mut DbPool<'_>,
    timestamp: DateTime<Utc>,
    form: &PostInsertForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post::table)
     .values(form)
     .on_conflict(post::ap_id)
     .filter_target(coalesce(post::updated_at, post::published_at).lt(timestamp))
     .do_update()
     .set(form)
     .get_result::<Self>(conn)
     .await
     .with_fastjob_type(FastJobErrorType::CouldntCreatePost)
  }


  pub async fn list_featured_for_category(
      pool: &mut DbPool<'_>,
      the_category_id: CategoryId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    post::table
      .filter(post::category_id.eq(the_category_id))
      .filter(post::deleted.eq(false))
      .filter(post::removed.eq(false))
      .filter(post::featured_category.eq(true))
      .then_order_by(post::published_at.desc())
      .limit(FETCH_LIMIT_MAX.try_into()?)
      .load::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn permadelete_for_creator(
    pool: &mut DbPool<'_>,
    for_creator_id: PersonId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    diesel::update(post::table.filter(post::creator_id.eq(for_creator_id)))
      .set((
        post::name.eq(DELETED_REPLACEMENT_TEXT),
        post::url.eq(Option::<&str>::None),
        post::body.eq(DELETED_REPLACEMENT_TEXT),
        post::deleted.eq(true),
        post::updated_at.eq(Utc::now()),
      ))
      .get_results::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdatePost)
  }

  async fn creator_post_ids_in_category(
      pool: &mut DbPool<'_>,
      creator_id: PersonId,
      category_id: CategoryId,
  ) -> FastJobResult<Vec<PostId>> {
    let conn = &mut get_conn(pool).await?;

    post::table
      .filter(post::creator_id.eq(creator_id))
      .filter(post::category_id.eq(category_id))
      .select(post::id)
      .load::<PostId>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  /// Diesel can't update from join unfortunately, so you sometimes need to fetch a list of post_ids
  /// for a creator.
  async fn creator_post_ids_in_instance(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    instance_id: InstanceId,
  ) -> FastJobResult<Vec<PostId>> {
    let conn = &mut get_conn(pool).await?;

    post::table
      .inner_join(category::table)
      .filter(post::creator_id.eq(creator_id))
      .filter(category::instance_id.eq(instance_id))
      .select(post::id)
      .load::<PostId>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn update_removed_for_creator_and_category(
      pool: &mut DbPool<'_>,
      creator_id: PersonId,
      category_id: CategoryId,
      removed: bool,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    update(post::table)
      .filter(post::creator_id.eq(creator_id))
      .filter(post::category_id.eq(category_id))
      .set((post::removed.eq(removed), post::updated_at.eq(Utc::now())))
      .get_results::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdatePost)
  }

  pub async fn update_removed_for_creator_and_instance(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    instance_id: InstanceId,
    removed: bool,
  ) -> FastJobResult<Vec<Self>> {
    let post_ids = Self::creator_post_ids_in_instance(pool, creator_id, instance_id).await?;

    let conn = &mut get_conn(pool).await?;

    update(post::table)
      .filter(post::id.eq_any(post_ids.clone()))
      .set((post::removed.eq(removed), post::updated_at.eq(Utc::now())))
      .get_results::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdatePost)
  }

  pub async fn update_removed_for_creator(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    removed: bool,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    update(post::table)
      .filter(post::creator_id.eq(creator_id))
      .set((post::removed.eq(removed), post::updated_at.eq(Utc::now())))
      .get_results::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdatePost)
  }

  pub fn is_post_creator(person_id: PersonId, post_creator_id: PersonId) -> bool {
    person_id == post_creator_id
  }

  pub async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: Url,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    let object_id: DbUrl = object_id.into();
    post::table
     .filter(post::ap_id.eq(object_id))
     .filter(post::scheduled_publish_time_at.is_null())
     .first(conn)
     .await
     .optional()
     .with_fastjob_type(FastJobErrorType::NotFound)
  }


  pub async fn user_scheduled_post_count(
    person_id: PersonId,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<i64> {
    let conn = &mut get_conn(pool).await?;

    post::table
      .inner_join(person::table)
      .inner_join(category::table)
      // find all posts which have scheduled_publish_time that is in the  future
      .filter(post::scheduled_publish_time_at.is_not_null())
      .filter(coalesce(post::scheduled_publish_time_at, now()).gt(now()))
      // make sure the post and category are still around
      .filter(not(post::deleted.or(post::removed)))
      .filter(not(category::removed.or(category::deleted)))
      // only posts by specified user
      .filter(post::creator_id.eq(person_id))
      .select(count(post::id))
      .first::<i64>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn update_ranks(pool: &mut DbPool<'_>, post_id: PostId) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    // Diesel can't update based on a join, which is necessary for the scaled_rank
    // https://github.com/diesel-rs/diesel/issues/1478
    // Just select the metrics we need manually, for now, since its a single post anyway

    // Use left_join since post.category_id is now nullable (for delivery posts)
    // Posts without categories will have 0 interactions_month for their scaled_rank
    let interactions_month = category::table
      .select(category::interactions_month)
      .left_join(post::table.on(category::id.nullable().eq(post::category_id)))
      .filter(post::id.eq(post_id))
      .filter(post::category_id.is_not_null()) // Only get interactions for posts with categories
      .first::<i64>(conn)
      .await
      .unwrap_or(0);

    diesel::update(post::table.find(post_id))
      .set((
        post::hot_rank.eq(hot_rank(post::score, post::published_at)),
        post::hot_rank_active.eq(hot_rank(post::score, post::newest_comment_time_necro_at)),
        post::scaled_rank.eq(scaled_rank(
          post::score,
          post::published_at,
          interactions_month,
        )),
      ))
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdatePost)
  }
  pub fn local_url(&self, settings: &Settings) -> FastJobResult<Url> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/post/{}", self.id))?)
  }
  pub async fn set_not_pending(&self, pool: &mut DbPool<'_>) -> FastJobResult<()> {
    if self.local && self.pending {
      let form = PostUpdateForm {
        pending: Some(false),
        ..Default::default()
      };
      Post::update(pool, self.id, &form).await?;
    }
    Ok(())
  }

  pub async fn check_post_name_taken(pool: &mut DbPool<'_>, name: &str) -> FastJobResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(not(exists(
      post::table
          .filter(lower(post::name).eq(name.to_lowercase()))
          .filter(post::local.eq(true)),
    )))
        .get_result::<bool>(conn)
        .await?
        .then_some(())
        .ok_or(FastJobErrorType::PostNameAlreadyExists.into())
  }
}

impl Likeable for PostActions {
  type Form = PostLikeForm;
  type IdType = PostId;

  async fn like(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    validate_like(form.like_score).with_fastjob_type(FastJobErrorType::CouldntLikePost)?;

    insert_into(post_actions::table)
      .values(form)
      .on_conflict((post_actions::post_id, post_actions::person_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntLikePost)
  }

  async fn remove_like(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    post_id: Self::IdType,
  ) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(post_actions::table.find((person_id, post_id)))
      .set_null(post_actions::like_score)
      .set_null(post_actions::liked_at)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntLikePost)
  }

  async fn remove_all_likes(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;

    uplete::new(post_actions::table.filter(post_actions::person_id.eq(person_id)))
      .set_null(post_actions::like_score)
      .set_null(post_actions::liked_at)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdatePost)
  }

  async fn remove_likes_in_category(
      pool: &mut DbPool<'_>,
      person_id: PersonId,
      category_id: CategoryId,
  ) -> FastJobResult<uplete::Count> {
    let post_ids = Post::creator_post_ids_in_category(pool, person_id, category_id).await?;

    let conn = &mut get_conn(pool).await?;

    uplete::new(post_actions::table.filter(post_actions::post_id.eq_any(post_ids.clone())))
      .set_null(post_actions::like_score)
      .set_null(post_actions::liked_at)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdatePost)
  }
}

impl Saveable for PostActions {
  type Form = PostSavedForm;
  async fn save(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_actions::table)
      .values(form)
      .on_conflict((post_actions::post_id, post_actions::person_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntSavePost)
  }
  async fn unsave(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(post_actions::table.find((form.person_id, form.post_id)))
      .set_null(post_actions::saved_at)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntSavePost)
  }
}

impl Readable for PostActions {
  type Form = PostReadForm;

  async fn mark_as_read(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<usize> {
    Self::mark_many_as_read(pool, std::slice::from_ref(form)).await
  }

  async fn mark_as_unread(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;

    uplete::new(
      post_actions::table
        .filter(post_actions::post_id.eq(form.post_id))
        .filter(post_actions::person_id.eq(form.person_id)),
    )
    .set_null(post_actions::read_at)
    .get_result(conn)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntMarkPostAsRead)
  }

  async fn mark_many_as_read(pool: &mut DbPool<'_>, forms: &[Self::Form]) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;

    insert_into(post_actions::table)
      .values(forms)
      .on_conflict((post_actions::person_id, post_actions::post_id))
      .do_update()
      .set(post_actions::read_at.eq(now().nullable()))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntMarkPostAsRead)
  }
}

impl Hideable for PostActions {
  type Form = PostHideForm;
  async fn hide(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(post_actions::table)
      .values(form)
      .on_conflict((post_actions::person_id, post_actions::post_id))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntHidePost)
  }

  async fn unhide(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;

    uplete::new(
      post_actions::table
        .filter(post_actions::post_id.eq(form.post_id))
        .filter(post_actions::person_id.eq(form.person_id)),
    )
    .set_null(post_actions::hidden_at)
    .get_result(conn)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntHidePost)
  }
}

impl ReadComments for PostActions {
  type Form = PostReadCommentsForm;
  type IdType = PostId;

  async fn update_read_comments(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(post_actions::table)
      .values(form)
      .on_conflict((post_actions::person_id, post_actions::post_id))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateReadComments)
  }

  async fn remove_read_comments(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    post_id: Self::IdType,
  ) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;

    uplete::new(
      post_actions::table
        .filter(post_actions::post_id.eq(post_id))
        .filter(post_actions::person_id.eq(person_id)),
    )
    .set_null(post_actions::read_comments_amount)
    .set_null(post_actions::read_comments_at)
    .get_result(conn)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntUpdateReadComments)
  }
}

impl PostActions {
  pub fn build_many_read_forms(post_ids: &[PostId], person_id: PersonId) -> Vec<PostReadForm> {
    post_ids
      .iter()
      .map(|post_id| PostReadForm::new(*post_id, person_id))
      .collect::<Vec<_>>()
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    person_id: PersonId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    post_actions::table
      .find((person_id, post_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn from_cursor(cursor: &PaginationCursor, pool: &mut DbPool<'_>) -> FastJobResult<Self> {
    let pids = cursor.prefixes_and_ids();
    let (_, person_id) = pids
      .as_slice()
      .first()
      .ok_or(FastJobErrorType::CouldntParsePaginationToken)?;
    let (_, post_id) = pids
      .get(1)
      .ok_or(FastJobErrorType::CouldntParsePaginationToken)?;
    Self::read(pool, PostId(*post_id), PersonId(*person_id)).await
  }

  pub async fn update_notification_state(
    post_id: PostId,
    person_id: PersonId,
    new_state: PostNotifications,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<PostActions> {
    let conn = &mut get_conn(pool).await?;
    let form = (
      post_actions::person_id.eq(person_id),
      post_actions::post_id.eq(post_id),
      post_actions::notifications.eq(new_state),
    );

    insert_into(post_actions::table)
      .values(form.clone())
      .on_conflict((post_actions::person_id, post_actions::post_id))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    source::{
        comment::{Comment, CommentInsertForm, CommentUpdateForm},
        category::{Category, CategoryInsertForm},
        instance::Instance,
        person::{Person, PersonInsertForm},
        post::{
        Post,
        PostActions,
        PostInsertForm,
        PostLikeForm,
        PostReadForm,
        PostSavedForm,
        PostUpdateForm,
      },
    },
    traits::{Crud, Likeable, Readable, Saveable},
    utils::{build_db_pool_for_tests, uplete, RANK_DEFAULT},
  };
  use chrono::DateTime;
  use app_108jobs_utils::error::FastJobResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;
  use url::Url;
  use crate::newtypes::DbUrl;

  #[tokio::test]
  #[serial]
  async fn test_crud() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "jim");

    let inserted_person = Person::create(pool, &new_person).await?;

    let new_category = CategoryInsertForm::new(
      inserted_instance.id,
      "test category_3".to_string(),
      "nada".to_owned(),
    );

    let inserted_category = Category::create(pool, &new_category).await?;

    let new_post = PostInsertForm::new(
      "A test post".into(),
      inserted_person.id,
      inserted_category.id,
    );
    let inserted_post = Post::create(pool, &new_post).await?;

    let new_post2 = PostInsertForm::new(
      "A test post 2".into(),
      inserted_person.id,
      inserted_category.id,
    );
    let inserted_post2 = Post::create(pool, &new_post2).await?;

    let new_scheduled_post = PostInsertForm {
      scheduled_publish_time_at: Some(DateTime::from_timestamp_nanos(i64::MAX)),
      ..PostInsertForm::new("beans".into(), inserted_person.id, inserted_category.id)
    };
    let inserted_scheduled_post = Post::create(pool, &new_scheduled_post).await?;

    let expected_post = Post {
      id: inserted_post.id,
      name: "A test post".into(),
      url: None,
      body: None,
      alt_text: None,
      creator_id: inserted_person.id,
      category_id: inserted_category.id,
      published_at: inserted_post.published_at,
      removed: false,
      locked: false,
      self_promotion: false,
      deleted: false,
      updated_at: None,
      embed_title: None,
      embed_description: None,
      embed_video_url: None,
      thumbnail_url: None,
      ap_id: Url::parse(&format!("https://108fasttob.com/post/{}", inserted_post.id))?.into(),
      local: true,
      language_id: Default::default(),
      featured_category: false,
      featured_local: false,
      url_content_type: None,
      scheduled_publish_time_at: None,
      comments: 0,
      controversy_rank: 0.0,
      downvotes: 0,
      upvotes: 1,
      score: 1,
      hot_rank: RANK_DEFAULT,
      hot_rank_active: RANK_DEFAULT,
      newest_comment_time_at: inserted_post.published_at,
      newest_comment_time_necro_at: inserted_post.published_at,
      report_count: 0,
      scaled_rank: RANK_DEFAULT,
      unresolved_report_count: 0,
      intended_use: Default::default(),
      job_type: Default::default(),
      budget: 0.0,
      deadline: None,
      is_english_required: false,
      pending: false,
    };

    // Post Like
    let post_like_form = PostLikeForm::new(inserted_post.id, inserted_person.id, 1);

    let inserted_post_like = PostActions::like(pool, &post_like_form).await?;
    assert_eq!(Some(1), inserted_post_like.like_score);

    // Post Save
    let post_saved_form = PostSavedForm::new(inserted_post.id, inserted_person.id);

    let inserted_post_saved = PostActions::save(pool, &post_saved_form).await?;
    assert!(inserted_post_saved.saved_at.is_some());

    // Mark 2 posts as read
    let post_read_form_1 = PostReadForm::new(inserted_post.id, inserted_person.id);
    PostActions::mark_as_read(pool, &post_read_form_1).await?;
    let post_read_form_2 = PostReadForm::new(inserted_post2.id, inserted_person.id);
    PostActions::mark_as_read(pool, &post_read_form_2).await?;

    let read_post = Post::read(pool, inserted_post.id).await?;

    let new_post_update = PostUpdateForm {
      name: Some("A test post".into()),
      ..Default::default()
    };
    let updated_post = Post::update(pool, inserted_post.id, &new_post_update).await?;

    // Scheduled post count
    let scheduled_post_count = Post::user_scheduled_post_count(inserted_person.id, pool).await?;
    assert_eq!(1, scheduled_post_count);

    let like_removed = PostActions::remove_like(pool, inserted_person.id, inserted_post.id).await?;
    assert_eq!(uplete::Count::only_updated(1), like_removed);
    let saved_removed = PostActions::unsave(pool, &post_saved_form).await?;
    assert_eq!(uplete::Count::only_updated(1), saved_removed);

    let read_remove_form_1 = PostReadForm::new(inserted_post.id, inserted_person.id);
    let read_removed_1 = PostActions::mark_as_unread(pool, &read_remove_form_1).await?;
    assert_eq!(uplete::Count::only_deleted(1), read_removed_1);

    let read_remove_form_2 = PostReadForm::new(inserted_post2.id, inserted_person.id);
    let read_removed_2 = PostActions::mark_as_unread(pool, &read_remove_form_2).await?;
    assert_eq!(uplete::Count::only_deleted(1), read_removed_2);

    let num_deleted = Post::delete(pool, inserted_post.id).await?
      + Post::delete(pool, inserted_post2.id).await?
      + Post::delete(pool, inserted_scheduled_post.id).await?;

    assert_eq!(3, num_deleted);
    Category::delete(pool, inserted_category.id).await?;
    Person::delete(pool, inserted_person.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    assert_eq!(expected_post, read_post);
    assert_eq!(expected_post, updated_post);

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_aggregates() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "thommy_category_agg");

    let inserted_person = Person::create(pool, &new_person).await?;

    let another_person = PersonInsertForm::test_form(inserted_instance.id, "jerry_category_agg");

    let another_inserted_person = Person::create(pool, &another_person).await?;

    let new_category = CategoryInsertForm::new(
      inserted_instance.id,
      "TIL_category_agg".into(),
      "nada".to_owned(),
    );
    let inserted_category = Category::create(pool, &new_category).await?;

    let new_post = PostInsertForm::new(
      "A test post".into(),
      inserted_person.id,
      inserted_category.id,
    );
    let inserted_post = Post::create(pool, &new_post).await?;

    let comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );
    let inserted_comment = Comment::create(pool, &comment_form).await?;

    let child_comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );
    let inserted_child_comment =
      Comment::create(pool, &child_comment_form).await?;

    let post_like = PostLikeForm::new(inserted_post.id, inserted_person.id, 1);

    PostActions::like(pool, &post_like).await?;

    let post_aggs_before_delete = Post::read(pool, inserted_post.id).await?;

    assert_eq!(2, post_aggs_before_delete.comments);
    assert_eq!(1, post_aggs_before_delete.score);
    assert_eq!(1, post_aggs_before_delete.upvotes);
    assert_eq!(0, post_aggs_before_delete.downvotes);

    // Add a post dislike from the other person
    let post_dislike = PostLikeForm::new(inserted_post.id, another_inserted_person.id, -1);

    PostActions::like(pool, &post_dislike).await?;

    let post_aggs_after_dislike = Post::read(pool, inserted_post.id).await?;

    assert_eq!(2, post_aggs_after_dislike.comments);
    assert_eq!(0, post_aggs_after_dislike.score);
    assert_eq!(1, post_aggs_after_dislike.upvotes);
    assert_eq!(1, post_aggs_after_dislike.downvotes);

    // Remove the comments
    Comment::delete(pool, inserted_comment.id).await?;
    Comment::delete(pool, inserted_child_comment.id).await?;
    let after_comment_delete = Post::read(pool, inserted_post.id).await?;
    assert_eq!(0, after_comment_delete.comments);
    assert_eq!(0, after_comment_delete.score);
    assert_eq!(1, after_comment_delete.upvotes);
    assert_eq!(1, after_comment_delete.downvotes);

    // Remove the first post like
    PostActions::remove_like(pool, inserted_person.id, inserted_post.id).await?;
    let after_like_remove = Post::read(pool, inserted_post.id).await?;
    assert_eq!(0, after_like_remove.comments);
    assert_eq!(-1, after_like_remove.score);
    assert_eq!(0, after_like_remove.upvotes);
    assert_eq!(1, after_like_remove.downvotes);

    // This should delete all the associated rows, and fire triggers
    Person::delete(pool, another_inserted_person.id).await?;
    let person_num_deleted = Person::delete(pool, inserted_person.id).await?;
    assert_eq!(1, person_num_deleted);

    // Delete the category
    let category_num_deleted = Category::delete(pool, inserted_category.id).await?;
    assert_eq!(1, category_num_deleted);

    // Should be none found, since the creator was deleted
    let after_delete = Post::read(pool, inserted_post.id).await;
    assert!(after_delete.is_err());

    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_aggregates_soft_delete() -> FastJobResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "thommy_category_agg");

    let inserted_person = Person::create(pool, &new_person).await?;

    let new_category = CategoryInsertForm::new(
      inserted_instance.id,
      "TIL_category_agg".into(),
      "nada".to_owned(),
    );
    let inserted_category = Category::create(pool, &new_category).await?;

    let new_post = PostInsertForm::new(
      "A test post".into(),
      inserted_person.id,
      inserted_category.id,
    );
    let inserted_post = Post::create(pool, &new_post).await?;

    let comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into()
    );

    let inserted_comment = Comment::create(pool, &comment_form).await?;

    let post_aggregates_before = Post::read(pool, inserted_post.id).await?;
    assert_eq!(1, post_aggregates_before.comments);

    Comment::update(
      pool,
      inserted_comment.id,
      &CommentUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let post_aggregates_after_remove = Post::read(pool, inserted_post.id).await?;
    assert_eq!(0, post_aggregates_after_remove.comments);

    Comment::update(
      pool,
      inserted_comment.id,
      &CommentUpdateForm {
        removed: Some(false),
        ..Default::default()
      },
    )
    .await?;

    Comment::update(
      pool,
      inserted_comment.id,
      &CommentUpdateForm {
        deleted: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let post_aggregates_after_delete = Post::read(pool, inserted_post.id).await?;
    assert_eq!(0, post_aggregates_after_delete.comments);

    Comment::update(
      pool,
      inserted_comment.id,
      &CommentUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let post_aggregates_after_delete_remove = Post::read(pool, inserted_post.id).await?;
    assert_eq!(0, post_aggregates_after_delete_remove.comments);

    Comment::delete(pool, inserted_comment.id).await?;
    Post::delete(pool, inserted_post.id).await?;
    Person::delete(pool, inserted_person.id).await?;
    Category::delete(pool, inserted_category.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }
}
