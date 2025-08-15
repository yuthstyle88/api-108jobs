#[cfg(feature = "full")]
use crate::{
  newtypes::JobBudgetPlanId,
  source::job_budget_plan::{JobBudgetPlan, JobBudgetPlanInsertForm, JobBudgetPlanUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};

#[cfg(feature = "full")]
use diesel::QueryDsl;
#[cfg(feature = "full")]
use diesel_async::RunQueryDsl;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::job_budget_plan;
#[cfg(feature = "full")]
use lemmy_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

#[cfg(feature = "full")]
impl Crud for JobBudgetPlan {
  type InsertForm = JobBudgetPlanInsertForm;
  type UpdateForm = JobBudgetPlanUpdateForm;
  type IdType = JobBudgetPlanId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(job_budget_plan::table)
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
    diesel::update(job_budget_plan::table.find(id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::DatabaseError)
  }
}
