use crate::{
  newtypes::{
    ModAddCategoryId,
    ModAddId,
    ModBanFromCategoryId,
    ModBanId,
    ModChangeCategoryVisibilityId,
    ModFeaturePostId,
    ModLockPostId,
    ModRemoveCommentId,
    ModRemoveCategoryId,
    ModRemovePostId,
    ModTransferCategoryId,
  },
  source::mod_log::moderator::{
    ModAdd,
    ModAddCategory,
    ModAddCategoryForm,
    ModAddForm,
    ModBan,
    ModBanForm,
    ModBanFromCategory,
    ModBanFromCategoryForm,
    ModChangeCategoryVisibility,
    ModChangeCategoryVisibilityForm,
    ModFeaturePost,
    ModFeaturePostForm,
    ModLockPost,
    ModLockPostForm,
    ModRemoveComment,
    ModRemoveCommentForm,
    ModRemoveCategory,
    ModRemoveCategoryForm,
    ModRemovePost,
    ModRemovePostForm,
    ModTransferCategory,
    ModTransferCategoryForm,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{
  mod_add,
  mod_add_category,
  mod_ban,
  mod_ban_from_category,
  mod_change_category_visibility,
  mod_feature_post,
  mod_lock_post,
  mod_remove_comment,
  mod_remove_category,
  mod_remove_post,
  mod_transfer_category,
};
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Crud for ModRemovePost {
  type InsertForm = ModRemovePostForm;
  type UpdateForm = ModRemovePostForm;
  type IdType = ModRemovePostId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_remove_post::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateModlog)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_remove_post::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateModlog)
  }
}

impl ModRemovePost {
  pub async fn create_multiple(
    pool: &mut DbPool<'_>,
    forms: &Vec<ModRemovePostForm>,
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_remove_post::table)
      .values(forms)
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateModlog)
  }
}

impl Crud for ModLockPost {
  type InsertForm = ModLockPostForm;
  type UpdateForm = ModLockPostForm;
  type IdType = ModLockPostId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_lock_post::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateModlog)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_lock_post::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateModlog)
  }
}

impl Crud for ModFeaturePost {
  type InsertForm = ModFeaturePostForm;
  type UpdateForm = ModFeaturePostForm;
  type IdType = ModFeaturePostId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_feature_post::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateModlog)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_feature_post::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateModlog)
  }
}

impl Crud for ModRemoveComment {
  type InsertForm = ModRemoveCommentForm;
  type UpdateForm = ModRemoveCommentForm;
  type IdType = ModRemoveCommentId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_remove_comment::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateModlog)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_remove_comment::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateModlog)
  }
}

impl ModRemoveComment {
  pub async fn create_multiple(
    pool: &mut DbPool<'_>,
    forms: &Vec<ModRemoveCommentForm>,
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_remove_comment::table)
      .values(forms)
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateModlog)
  }
}

impl Crud for ModRemoveCategory {
  type InsertForm = ModRemoveCategoryForm;
  type UpdateForm = ModRemoveCategoryForm;
  type IdType = ModRemoveCategoryId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_remove_category::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateModlog)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_remove_category::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateModlog)
  }
}

impl Crud for ModBanFromCategory {
  type InsertForm = ModBanFromCategoryForm;
  type UpdateForm = ModBanFromCategoryForm;
  type IdType = ModBanFromCategoryId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_ban_from_category::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateModlog)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_ban_from_category::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateModlog)
  }
}

impl Crud for ModBan {
  type InsertForm = ModBanForm;
  type UpdateForm = ModBanForm;
  type IdType = ModBanId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_ban::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateModlog)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_ban::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateModlog)
  }
}

impl Crud for ModChangeCategoryVisibility {
  type InsertForm = ModChangeCategoryVisibilityForm;
  type UpdateForm = ModChangeCategoryVisibilityForm;
  type IdType = ModChangeCategoryVisibilityId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_change_category_visibility::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateModlog)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_change_category_visibility::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateModlog)
  }
}

impl Crud for ModAddCategory {
  type InsertForm = ModAddCategoryForm;
  type UpdateForm = ModAddCategoryForm;
  type IdType = ModAddCategoryId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_add_category::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateModlog)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_add_category::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateModlog)
  }
}

impl Crud for ModTransferCategory {
  type InsertForm = ModTransferCategoryForm;
  type UpdateForm = ModTransferCategoryForm;
  type IdType = ModTransferCategoryId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_transfer_category::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateModlog)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_transfer_category::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateModlog)
  }
}

impl Crud for ModAdd {
  type InsertForm = ModAddForm;
  type UpdateForm = ModAddForm;
  type IdType = ModAddId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_add::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateModlog)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    from_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(mod_add::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateModlog)
  }
}
