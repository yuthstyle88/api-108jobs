use crate::{
    newtypes::{CategoryId, CategoryReportId, PersonId},
    source::category_report::{CategoryReport, CategoryReportForm},
    traits::Reportable,
    utils::{get_conn, DbPool},
};
use chrono::Utc;
use diesel::{
  dsl::{insert_into, update},
  BoolExpressionMethods,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use app_108jobs_db_schema_file::schema::category_report;
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl Reportable for CategoryReport {
  type Form = CategoryReportForm;
  type IdType = CategoryReportId;
  type ObjectIdType = CategoryId;
  /// creates a category report and returns it
  ///
  /// * `conn` - the postgres connection
  /// * `category_report_form` - the filled categoryReportForm to insert
  async fn report(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(category_report::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateReport)
  }

  /// resolve a category report
  ///
  /// * `conn` - the postgres connection
  /// * `report_id` - the id of the report to resolve
  /// * `by_resolver_id` - the id of the user resolving the report
  async fn resolve(
    pool: &mut DbPool<'_>,
    report_id_: Self::IdType,
    by_resolver_id: PersonId,
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(category_report::table.find(report_id_))
      .set((
        category_report::resolved.eq(true),
        category_report::resolver_id.eq(by_resolver_id),
        category_report::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntResolveReport)
  }

  async fn resolve_apub(
    pool: &mut DbPool<'_>,
    object_id: Self::ObjectIdType,
    report_creator_id: PersonId,
    resolver_id: PersonId,
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(
      category_report::table.filter(
        category_report::category_id
          .eq(object_id)
          .and(category_report::creator_id.eq(report_creator_id)),
      ),
    )
    .set((
      category_report::resolved.eq(true),
      category_report::resolver_id.eq(resolver_id),
      category_report::updated_at.eq(Utc::now()),
    ))
    .execute(conn)
    .await
    .with_fastjob_type(FastJobErrorType::CouldntResolveReport)
  }

  async fn resolve_all_for_object(
    pool: &mut DbPool<'_>,
    category_id_: Self::ObjectIdType,
    by_resolver_id: PersonId,
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(category_report::table.filter(category_report::category_id.eq(category_id_)))
      .set((
        category_report::resolved.eq(true),
        category_report::resolver_id.eq(by_resolver_id),
        category_report::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntResolveReport)
  }

  /// unresolve a category report
  ///
  /// * `conn` - the postgres connection
  /// * `report_id` - the id of the report to unresolve
  /// * `by_resolver_id` - the id of the user unresolving the report
  async fn unresolve(
    pool: &mut DbPool<'_>,
    report_id_: Self::IdType,
    by_resolver_id: PersonId,
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(category_report::table.find(report_id_))
      .set((
        category_report::resolved.eq(false),
        category_report::resolver_id.eq(by_resolver_id),
        category_report::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntResolveReport)
  }
}
