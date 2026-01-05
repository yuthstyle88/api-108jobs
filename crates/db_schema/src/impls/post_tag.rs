use crate::{
  diesel::SelectableHelper,
  newtypes::{PostId, TagId},
  source::post_tag::{PostTag, PostTagForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{delete, insert_into, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use app_108jobs_db_schema_file::schema::post_tag;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl PostTag {
  pub async fn set(pool: &mut DbPool<'_>, tags: &[PostTagForm]) -> FastJobResult<Vec<Self>> {
    let post_id = tags.first().map(|t| t.post_id).unwrap_or_default();
    PostTag::delete_for_post(pool, post_id).await?;
    PostTag::create_many(pool, tags).await
  }

  async fn delete_for_post(pool: &mut DbPool<'_>, post_id: PostId) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    delete(post_tag::table.filter(post_tag::post_id.eq(post_id)))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::Deleted)
  }

  async fn create_many(pool: &mut DbPool<'_>, forms: &[PostTagForm]) -> FastJobResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_tag::table)
      .values(forms)
      .returning(Self::as_select())
      .get_results(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreatePostTag)
  }
}

impl Crud for PostTag {
  type InsertForm = PostTagForm;
  type UpdateForm = PostTagForm;
  type IdType = (PostId, TagId);

  async fn create(pool: &mut DbPool<'_>, form: &PostTagForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_tag::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreatePostTag)
  }

  async fn update(
    _pool: &mut DbPool<'_>,
    _id: Self::IdType,
    _form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    Err(FastJobErrorType::CouldntUpdatePostTag.into())
  }
}
