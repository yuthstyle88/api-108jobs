use crate::{
  newtypes::{
    ModAddCommunityId,
    ModAddId,
    ModBanFromCommunityId,
    ModBanId,
    ModChangeCommunityVisibilityId,
    ModFeaturePostId,
    ModLockPostId,
    ModRemoveCommentId,
    ModRemoveCommunityId,
    ModRemovePostId,
    ModTransferCommunityId,
  },
  source::mod_log::moderator::{
    ModAdd,
    ModAddCommunity,
    ModAddCommunityForm,
    ModAddForm,
    ModBan,
    ModBanForm,
    ModBanFromCommunity,
    ModBanFromCommunityForm,
    ModChangeCommunityVisibility,
    ModChangeCommunityVisibilityForm,
    ModFeaturePost,
    ModFeaturePostForm,
    ModLockPost,
    ModLockPostForm,
    ModRemoveComment,
    ModRemoveCommentForm,
    ModRemoveCommunity,
    ModRemoveCommunityForm,
    ModRemovePost,
    ModRemovePostForm,
    ModTransferCommunity,
    ModTransferCommunityForm,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{
  mod_add,
  mod_add_community,
  mod_ban,
  mod_ban_from_community,
  mod_change_community_visibility,
  mod_feature_post,
  mod_lock_post,
  mod_remove_comment,
  mod_remove_community,
  mod_remove_post,
  mod_transfer_community,
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

impl Crud for ModRemoveCommunity {
  type InsertForm = ModRemoveCommunityForm;
  type UpdateForm = ModRemoveCommunityForm;
  type IdType = ModRemoveCommunityId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_remove_community::table)
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
    diesel::update(mod_remove_community::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateModlog)
  }
}

impl Crud for ModBanFromCommunity {
  type InsertForm = ModBanFromCommunityForm;
  type UpdateForm = ModBanFromCommunityForm;
  type IdType = ModBanFromCommunityId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_ban_from_community::table)
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
    diesel::update(mod_ban_from_community::table.find(from_id))
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

impl Crud for ModChangeCommunityVisibility {
  type InsertForm = ModChangeCommunityVisibilityForm;
  type UpdateForm = ModChangeCommunityVisibilityForm;
  type IdType = ModChangeCommunityVisibilityId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_change_community_visibility::table)
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
    diesel::update(mod_change_community_visibility::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateModlog)
  }
}

impl Crud for ModAddCommunity {
  type InsertForm = ModAddCommunityForm;
  type UpdateForm = ModAddCommunityForm;
  type IdType = ModAddCommunityId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_add_community::table)
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
    diesel::update(mod_add_community::table.find(from_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntUpdateModlog)
  }
}

impl Crud for ModTransferCommunity {
  type InsertForm = ModTransferCommunityForm;
  type UpdateForm = ModTransferCommunityForm;
  type IdType = ModTransferCommunityId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(mod_transfer_community::table)
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
    diesel::update(mod_transfer_community::table.find(from_id))
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
