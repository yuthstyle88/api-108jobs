use crate::{
  newtypes::{PersonId, ProposalId, ProposalReportId},
  schema::proposal_report,
  source::proposal_report::{ProposalReport, ProposalReportForm},
  traits::Reportable,
  utils::{get_conn, DbPool},
};
use app_108jobs_core::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use chrono::Utc;
use diesel::{
  dsl::{insert_into, update},
  ExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;

impl Reportable for ProposalReport {
  type Form = ProposalReportForm;
  type IdType = ProposalReportId;
  type ObjectIdType = ProposalId;
  /// creates a proposal report and returns it
  ///
  /// * `conn` - the postgres connection
  /// * `comment_report_form` - the filled ProposalReportForm to insert
  async fn report(pool: &mut DbPool<'_>, form: &Self::Form) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(proposal_report::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntCreateReport)
  }

  /// resolve a proposal report
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
    update(proposal_report::table.find(report_id_))
      .set((
        proposal_report::resolved.eq(true),
        proposal_report::resolver_id.eq(by_resolver_id),
        proposal_report::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntResolveReport)
  }

  async fn resolve_all_for_object(
    pool: &mut DbPool<'_>,
    proposal_id_: ProposalId,
    by_resolver_id: PersonId,
  ) -> FastJobResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(proposal_report::table.filter(proposal_report::comment_id.eq(proposal_id_)))
      .set((
        proposal_report::resolved.eq(true),
        proposal_report::resolver_id.eq(by_resolver_id),
        proposal_report::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntResolveReport)
  }

  /// unresolve a proposal report
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
    update(proposal_report::table.find(report_id_))
      .set((
        proposal_report::resolved.eq(false),
        proposal_report::resolver_id.eq(by_resolver_id),
        proposal_report::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await
      .with_fastjob_type(FastJobErrorType::CouldntResolveReport)
  }
}
