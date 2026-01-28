use crate::{
  diesel::{OptionalExtension, NullableExpressionMethods},
  newtypes::{CommentId, CategoryId, DbUrl, InstanceId, PersonId},
  source::comment::{
    Comment, CommentActions, CommentInsertForm, CommentLikeForm, CommentSavedForm,
    CommentUpdateForm,
  },
  traits::{Crud, Likeable, Saveable},
  utils::{functions::hot_rank, get_conn, uplete, validate_like, DbPool, DELETED_REPLACEMENT_TEXT},
};
use chrono::Utc;
use diesel::{
  dsl::insert_into, expression::SelectableHelper, update, ExpressionMethods, JoinOnDsl, QueryDsl,
};
use diesel_async::RunQueryDsl;
use app_108jobs_db_schema_file::schema::{comment, comment_actions, category, post};
use app_108jobs_utils::{
  error::{FastJobErrorExt, FastJobErrorExt2, FastJobErrorType, FastJobResult},
  settings::structs::Settings,
};
use url::Url;

impl Crud for Comment {
  type InsertForm = CommentInsertForm;
  type UpdateForm = CommentUpdateForm;
  type IdType = CommentId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(comment::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateComment)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    update(comment::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateComment)
  }
}

impl Comment {
  pub async fn permadelete_for_creator(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    diesel::update(comment::table.filter(comment::creator_id.eq(creator_id)))
      .set((
        comment::content.eq(DELETED_REPLACEMENT_TEXT),
        comment::deleted.eq(true),
        comment::updated_at.eq(Utc::now()),
      ))
      .get_results::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateComment)
  }

  pub async fn update_removed_for_creator(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    removed: bool,
  ) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    update(comment::table.filter(comment::creator_id.eq(creator_id)))
      .set((
        comment::removed.eq(removed),
        comment::updated_at.eq(Utc::now()),
      ))
      .get_results::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateComment)
  }

  /// Diesel can't update from join unfortunately, so you'll need to loop over these
  async fn creator_comment_ids_in_category(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    category_id: CategoryId,
  ) -> FastJobResult<Vec<CommentId>> {
    let conn = &mut get_conn(pool).await?;

    comment::table
      .inner_join(post::table)
      .filter(comment::creator_id.eq(creator_id))
      .filter(post::category_id.eq(category_id))
      .select(comment::id)
      .load::<CommentId>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  /// Diesel can't update from join unfortunately, so you'll need to loop over these
  async fn creator_comment_ids_in_instance(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    instance_id: InstanceId,
  ) -> FastJobResult<Vec<CommentId>> {
    let conn = &mut get_conn(pool).await?;
    // Use nullable().eq() to compare nullable post.category_id with category.id
    let category_join = category::table.on(category::id.nullable().eq(post::category_id));

    comment::table
      .inner_join(post::table)
      .inner_join(category_join)
      .filter(comment::creator_id.eq(creator_id))
      .filter(post::category_id.is_not_null()) // Only include comments on posts with categories
      .filter(category::instance_id.eq(instance_id))
      .select(comment::id)
      .load::<CommentId>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn update_removed_for_creator_and_category(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    category_id: CategoryId,
    removed: bool,
  ) -> FastJobResult<Vec<CommentId>> {
    let comment_ids =
      Self::creator_comment_ids_in_category(pool, creator_id, category_id).await?;

    let conn = &mut get_conn(pool).await?;

    update(comment::table)
      .filter(comment::id.eq_any(comment_ids.clone()))
      .set((
        comment::removed.eq(removed),
        comment::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await?;

    Ok(comment_ids)
  }

  pub async fn update_removed_for_creator_and_instance(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    instance_id: InstanceId,
    removed: bool,
  ) -> FastJobResult<Vec<CommentId>> {
    let comment_ids = Self::creator_comment_ids_in_instance(pool, creator_id, instance_id).await?;
    let conn = &mut get_conn(pool).await?;

    update(comment::table)
      .filter(comment::id.eq_any(comment_ids.clone()))
      .set((
        comment::removed.eq(removed),
        comment::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await?;
    Ok(comment_ids)
  }

  pub async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: Url,
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    let object_id: DbUrl = object_id.into();
    comment::table
      .filter(comment::ap_id.eq(object_id))
      .first(conn)
      .await
      .optional()
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub fn parent_comment_id(&self) -> Option<CommentId> {
    let mut ltree_split: Vec<&str> = self.path.0.split('.').collect();
    ltree_split.remove(0); // The first is always 0
    if ltree_split.len() > 1 {
      let parent_comment_id = ltree_split.get(ltree_split.len() - 2);
      parent_comment_id.and_then(|p| p.parse::<i32>().map(CommentId).ok())
    } else {
      None
    }
  }
  pub async fn update_hot_rank(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    update(comment::table.find(comment_id))
      .set(comment::hot_rank.eq(hot_rank(comment::score, comment::published_at)))
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateComment)
  }

  pub fn local_url(&self, settings: &Settings) -> FastJobResult<Url> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/comment/{}", self.id))?)
  }

  pub async fn update_ap_id(
    pool: &mut DbPool<'_>,
    id: CommentId,
    ap_id: DbUrl,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    update(comment::table.find(id))
      .set(comment::ap_id.eq(ap_id))
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateComment)
  }

  /// The comment was created locally and sent back, indicating that the category accepted it
  pub async fn set_not_pending(&self, pool: &mut DbPool<'_>) -> FastJobResult<()> {
    if self.local && self.pending {
      let form = CommentUpdateForm {
        pending: Some(false),
        ..Default::default()
      };
      Comment::update(pool, self.id, &form).await?;
    }
    Ok(())
  }

  pub fn generate_comment_url(name: &str, settings: &Settings) -> FastJobResult<DbUrl> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/comment/{name}"))?.into())
  }
}

impl Likeable for CommentActions {
  type Form = CommentLikeForm;
  type IdType = CommentId;

  async fn like(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    validate_like(form.like_score).with_fastjob_type(FastJobErrorType::CouldntLikeComment)?;

    insert_into(comment_actions::table)
      .values(form)
      .on_conflict((comment_actions::comment_id, comment_actions::person_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntLikeComment)
  }
  async fn remove_like(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    comment_id: Self::IdType,
  ) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(comment_actions::table.find((person_id, comment_id)))
      .set_null(comment_actions::like_score)
      .set_null(comment_actions::liked_at)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntLikeComment)
  }

  async fn remove_all_likes(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
  ) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;

    uplete::new(comment_actions::table.filter(comment_actions::person_id.eq(creator_id)))
      .set_null(comment_actions::like_score)
      .set_null(comment_actions::liked_at)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateComment)
  }

  async fn remove_likes_in_category(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    category_id: CategoryId,
  ) -> FastJobResult<uplete::Count> {
    let comment_ids =
      Comment::creator_comment_ids_in_category(pool, creator_id, category_id).await?;

    let conn = &mut get_conn(pool).await?;

    uplete::new(
      comment_actions::table.filter(comment_actions::comment_id.eq_any(comment_ids.clone())),
    )
    .set_null(comment_actions::like_score)
    .set_null(comment_actions::liked_at)
    .get_result(conn)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntUpdateComment)
  }
}

impl Saveable for CommentActions {
  type Form = CommentSavedForm;
  async fn save(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(comment_actions::table)
      .values(form)
      .on_conflict((comment_actions::comment_id, comment_actions::person_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntSaveComment)
  }
  async fn unsave(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(comment_actions::table.find((form.person_id, form.comment_id)))
      .set_null(comment_actions::saved_at)
      .get_result(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntSaveComment)
  }
}

impl CommentActions {
  pub async fn read(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
    person_id: PersonId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    comment_actions::table
      .find((person_id, comment_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}
