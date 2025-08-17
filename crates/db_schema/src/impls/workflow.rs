use crate::newtypes::{PostId, WorkflowId};
#[cfg(feature = "full")]
use crate::{
  source::workflow::{Workflow, WorkflowInsertForm, WorkflowUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
#[cfg(feature = "full")]
use diesel::QueryDsl;
use diesel::{ExpressionMethods, OptionalExtension};
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::workflow;
use lemmy_db_schema_file::schema::workflow::dsl as wf;
#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

#[cfg(feature = "full")]
impl Crud for Workflow {
  type InsertForm = WorkflowInsertForm;
  type UpdateForm = WorkflowUpdateForm;
  type IdType = WorkflowId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(workflow::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(workflow::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}

#[cfg(feature = "full")]
impl Workflow {
  pub async fn get_by_post_id(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    seq_number: i16
  ) -> FastJobResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    wf::workflow
      .filter(wf::post_id.eq(post_id))
      .filter(wf::seq_number.eq(seq_number))
      .first::<Self>(conn)
      .await
      .optional()
      .with_fastjob_type(FastJobErrorType::NotFound)
  }

  pub async fn upsert_default(pool: &mut DbPool<'_>, post_id: PostId, seq_number: i16) -> FastJobResult<Self> {
    if let Some(w) = Self::get_by_post_id(pool, post_id, seq_number).await? {
      return Ok(w);
    }
    let form = WorkflowInsertForm::new(post_id, seq_number);
    Self::create(pool, &form).await
  }
  pub async fn delete_by_post(pool: &mut DbPool<'_>, post_id: PostId) -> FastJobResult<()> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    let conn = &mut get_conn(pool).await?;
    diesel::delete(wf::workflow.filter(wf::post_id.eq(post_id)))
      .execute(conn)
      .await?;
    Ok(())
  }
}
