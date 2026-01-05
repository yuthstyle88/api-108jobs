use crate::CategoryReportView;
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use app_108jobs_db_schema::{
  aliases,
  newtypes::{CategoryReportId, PersonId},
  utils::{get_conn, DbPool},
};
use app_108jobs_db_schema_file::schema::{category, category_actions, category_report, person};
use app_108jobs_utils::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};

impl CategoryReportView {
  /// returns the categoryReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub async fn read(
    pool: &mut DbPool<'_>,
    report_id: CategoryReportId,
    my_person_id: PersonId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;

    let resolver_id = aliases::person2.field(person::id);

    let report_creator_join = person::table.on(category_report::creator_id.eq(person::id));
    let resolver_join =
      aliases::person2.on(category_report::resolver_id.eq(resolver_id.nullable()));

    let category_actions_join = category_actions::table.on(
        category_actions::category_id
        .eq(category_report::category_id)
        .and(category_actions::person_id.eq(my_person_id)),
    );

    category_report::table
      .find(report_id)
      .inner_join(category::table)
      .inner_join(report_creator_join)
      .left_join(resolver_join)
      .left_join(category_actions_join)
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}
