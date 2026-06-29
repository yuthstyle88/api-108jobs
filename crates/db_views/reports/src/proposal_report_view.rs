use crate::ProposalReportView;
use app_108jobs_core::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use app_108jobs_db::{
  aliases::{self, creator_category_actions},
  newtypes::{PersonId, ProposalReportId},
  schema::{
    category,
    category_actions,
    local_user,
    person,
    person_actions,
    post,
    proposal,
    proposal_actions,
    proposal_report,
  },
  utils::{get_conn, DbPool},
};
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;

impl ProposalReportView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: PersonId) -> _ {
    let recipient_id = aliases::person1.field(person::id);
    let resolver_id = aliases::person2.field(person::id);

    let post_join = post::table.on(proposal::post_id.eq(post::id));

    let category_join = category::table.on(category::id.nullable().eq(post::category_id));

    let report_creator_join = person::table.on(proposal_report::creator_id.eq(person::id));

    let local_user_join = local_user::table.on(
      proposal::creator_id
        .eq(local_user::person_id)
        .and(local_user::admin.eq(true)),
    );

    let comment_creator_join = aliases::person1.on(proposal::creator_id.eq(recipient_id));

    let proposal_actions_join = proposal_actions::table.on(
      proposal_actions::proposal_id
        .eq(proposal_report::comment_id)
        .and(proposal_actions::person_id.eq(my_person_id)),
    );

    let resolver_join =
      aliases::person2.on(proposal_report::resolver_id.eq(resolver_id.nullable()));

    let creator_category_actions_join = creator_category_actions.on(
      creator_category_actions
        .field(category_actions::category_id)
        .nullable()
        .eq(post::category_id)
        .and(
          creator_category_actions
            .field(category_actions::person_id)
            .eq(proposal::creator_id),
        ),
    );

    let person_actions_join = person_actions::table.on(
      person_actions::target_id
        .eq(proposal::creator_id)
        .and(person_actions::person_id.eq(my_person_id)),
    );

    let category_actions_join = category_actions::table.on(
      category_actions::category_id
        .nullable()
        .eq(post::category_id)
        .and(category_actions::person_id.eq(my_person_id)),
    );

    proposal_report::table
      .inner_join(proposal::table)
      .inner_join(post_join)
      .left_join(category_join)
      .inner_join(report_creator_join)
      .inner_join(comment_creator_join)
      .left_join(proposal_actions_join)
      .left_join(resolver_join)
      .left_join(creator_category_actions_join)
      .left_join(local_user_join)
      .left_join(person_actions_join)
      .left_join(category_actions_join)
  }

  /// returns the ProposalReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub async fn read(
    pool: &mut DbPool<'_>,
    report_id: ProposalReportId,
    my_person_id: PersonId,
  ) -> FastJobResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins(my_person_id)
      .filter(proposal_report::id.eq(report_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}
