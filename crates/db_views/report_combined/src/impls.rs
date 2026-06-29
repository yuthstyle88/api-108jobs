use crate::{
  CategoryReportView, LocalUserView, PostReportView, ProposalReportView, ReportCombinedView,
  ReportCombinedViewInternal,
};
use app_108jobs_core::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use app_108jobs_db::{
  aliases::{self, creator_category_actions},
  newtypes::{CategoryId, PaginationCursor, PersonId, PostId},
  schema::{
    category, category_actions, category_report, local_user, person, person_actions, post,
    post_actions, post_report, proposal, proposal_actions, proposal_report, report_combined,
  },
  source::combined::report::{report_combined_keys as key, ReportCombined},
  traits::{InternalToCombinedView, PaginationCursorBuilder},
  utils::{get_conn, limit_fetch, paginate, DbPool},
  ReportType,
};
use chrono::{DateTime, Days, Utc};
use diesel::{
  BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
  PgExpressionMethods, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::asc_if;

impl ReportCombinedViewInternal {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: PersonId) -> _ {
    let report_creator = person::id;
    let item_creator = aliases::person1.field(person::id);

    let comment_join = proposal::table.on(proposal_report::comment_id.eq(proposal::id));
    let post_join = post::table.on(
      post_report::post_id
        .eq(post::id)
        .or(proposal::post_id.eq(post::id)),
    );

    let category_actions_join = category_actions::table.on(
      category_actions::category_id
        .nullable()
        .eq(post::category_id)
        .and(category_actions::person_id.eq(my_person_id)),
    );

    let report_creator_join = person::table.on(
      post_report::creator_id
        .eq(report_creator)
        .or(proposal_report::creator_id.eq(report_creator))
        .or(category_report::creator_id.eq(report_creator)),
    );

    let item_creator_join = aliases::person1.on(
      post::creator_id
        .eq(item_creator)
        .or(proposal::creator_id.eq(item_creator)),
    );

    let category_join = category::table.on(
      category_report::category_id
        .eq(category::id)
        .or(category::id.nullable().eq(post::category_id)),
    );

    let local_user_join = local_user::table.on(
      item_creator
        .eq(local_user::person_id)
        .and(local_user::admin.eq(true)),
    );

    let creator_category_actions_join = creator_category_actions.on(
      creator_category_actions
        .field(category_actions::category_id)
        .nullable()
        .eq(post::category_id)
        .and(
          creator_category_actions
            .field(category_actions::person_id)
            .eq(item_creator),
        ),
    );

    let post_actions_join = post_actions::table.on(
      post_actions::post_id
        .eq(post::id)
        .and(post_actions::person_id.eq(my_person_id)),
    );

    let person_actions_join = person_actions::table.on(
      person_actions::target_id
        .eq(item_creator)
        .and(person_actions::person_id.eq(my_person_id)),
    );

    let proposal_actions_join = proposal_actions::table.on(
      proposal_actions::proposal_id
        .eq(proposal::id)
        .and(proposal_actions::person_id.eq(my_person_id)),
    );

    report_combined::table
      .left_join(post_report::table)
      .left_join(proposal_report::table)
      .left_join(category_report::table)
      .inner_join(report_creator_join)
      .left_join(comment_join)
      .left_join(post_join)
      .left_join(item_creator_join)
      .left_join(category_join)
      .left_join(creator_category_actions_join)
      .left_join(local_user_join)
      .left_join(category_actions_join)
      .left_join(post_actions_join)
      .left_join(person_actions_join)
      .left_join(proposal_actions_join)
  }

  /// returns the current unresolved report count for the communities you mod
  pub async fn get_report_count(
    pool: &mut DbPool<'_>,
    user: &LocalUserView,
    category_id: Option<CategoryId>,
  ) -> FastJobResult<i64> {
    use diesel::dsl::count;

    let conn = &mut get_conn(pool).await?;
    let my_person_id = user.local_user.person_id;

    let mut query = Self::joins(my_person_id)
      .filter(report_is_not_resolved())
      .select(count(report_combined::id))
      .into_boxed();

    if let Some(category_id) = category_id {
      query = query.filter(
        category::id
          .eq(category_id)
          .and(report_combined::category_report_id.is_null()),
      );
    }

    if user.local_user.admin {
      query = query.filter(filter_admin_reports(Utc::now() - Days::new(3)));
    } else {
      query = query.filter(filter_mod_reports());
    }

    query
      .first::<i64>(conn)
      .await
      .with_fastjob_type(FastJobErrorType::NotFound)
  }
}

impl PaginationCursorBuilder for ReportCombinedView {
  type CursorData = ReportCombined;

  fn to_cursor(&self) -> PaginationCursor {
    let (prefix, id) = match &self {
      ReportCombinedView::Proposal(v) => ('C', v.proposal_report.id.0),
      ReportCombinedView::Post(v) => ('P', v.post_report.id.0),
      ReportCombinedView::Category(v) => ('Y', v.category_report.id.0),
    };
    PaginationCursor::new_single(prefix, id)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> FastJobResult<Self::CursorData> {
    let conn = &mut get_conn(pool).await?;
    let pids = cursor.prefixes_and_ids();
    let (prefix, id) = pids
      .as_slice()
      .first()
      .ok_or(FastJobErrorType::CouldntParsePaginationToken)?;

    let mut query = report_combined::table
      .select(Self::CursorData::as_select())
      .into_boxed();

    query = match prefix {
      'C' => query.filter(report_combined::proposal_report_id.eq(id)),
      'P' => query.filter(report_combined::post_report_id.eq(id)),
      'Y' => query.filter(report_combined::category_report_id.eq(id)),
      _ => return Err(FastJobErrorType::CouldntParsePaginationToken.into()),
    };
    let token = query.first(conn).await?;

    Ok(token)
  }
}

#[derive(Default)]
pub struct ReportCombinedQuery {
  pub type_: Option<ReportType>,
  pub post_id: Option<PostId>,
  pub category_id: Option<CategoryId>,
  pub unresolved_only: Option<bool>,
  /// For admins, also show reports with `violates_instance_rules=false`
  pub show_category_rule_violations: Option<bool>,
  pub cursor_data: Option<ReportCombined>,
  pub my_reports_only: Option<bool>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

impl ReportCombinedQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: &LocalUserView,
  ) -> FastJobResult<Vec<ReportCombinedView>> {
    let my_person_id = user.local_user.person_id;

    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(self.limit)?;
    let mut query = ReportCombinedViewInternal::joins(my_person_id)
      .select(ReportCombinedViewInternal::as_select())
      .limit(limit)
      .into_boxed();

    if let Some(category_id) = self.category_id {
      query = query.filter(
        category::id
          .eq(category_id)
          .and(report_combined::category_report_id.is_null()),
      );
    }

    if user.local_user.admin {
      let show_category_rule_violations = self.show_category_rule_violations.unwrap_or_default();
      if !show_category_rule_violations {
        query = query.filter(filter_admin_reports(Utc::now() - Days::new(3)));
      }
    } else {
      query = query.filter(filter_mod_reports());
    }

    if let Some(post_id) = self.post_id {
      query = query.filter(post::id.eq(post_id));
    }

    if self.my_reports_only.unwrap_or_default() {
      query = query.filter(person::id.eq(my_person_id));
    }

    if let Some(type_) = self.type_ {
      query = match type_ {
        ReportType::All => query,
        ReportType::Posts => query.filter(report_combined::post_report_id.is_not_null()),
        ReportType::Proposals => query.filter(report_combined::proposal_report_id.is_not_null()),
        ReportType::Communities => query.filter(report_combined::category_report_id.is_not_null()),
      }
    }

    // If viewing all reports, order by newest, but if viewing unresolved only, show the oldest
    // first (FIFO)
    let unresolved_only = self.unresolved_only.unwrap_or_default();
    let sort_direction = asc_if(unresolved_only);

    if unresolved_only {
      query = query.filter(report_is_not_resolved())
    };

    // Sorting by published
    let paginated_query = paginate(
      query,
      sort_direction,
      self.cursor_data,
      None,
      self.page_back,
    )
    .then_order_by(key::published_at)
    // Tie breaker
    .then_order_by(key::id);

    let res = paginated_query
      .load::<ReportCombinedViewInternal>(conn)
      .await?;

    // Map the query results to the enum
    let out = res
      .into_iter()
      .filter_map(InternalToCombinedView::map_to_enum)
      .collect();

    Ok(out)
  }
}

/// Mods can only see reports for posts/comments inside communities where they are moderator,
/// and which have `violates_instance_rules == false`.
#[diesel::dsl::auto_type]
fn filter_mod_reports() -> _ {
  category_actions::became_moderator_at
    .is_not_null()
    // Reporting a category or private message must go to admins
    .and(report_combined::category_report_id.is_null())
    .and(filter_violates_instance_rules().is_distinct_from(true))
}

/// Admins can see reports intended for them, or mod reports older than 3 days. Also reports
/// on communities, person and private messages.
#[diesel::dsl::auto_type]
fn filter_admin_reports(interval: DateTime<Utc>) -> _ {
  filter_violates_instance_rules()
    .or(report_combined::published_at.lt(interval))
    // Also show category reports where the admin is a category mod
    .or(category_actions::became_moderator_at.is_not_null())
}

/// Filter reports which are only for admins (either post/proposal report with
/// `violates_instance_rules=true`, or report on a category/person/private message.
#[diesel::dsl::auto_type]
fn filter_violates_instance_rules() -> _ {
  post_report::violates_instance_rules
    .or(proposal_report::violates_instance_rules)
    .or(report_combined::category_report_id.is_not_null())
}

#[diesel::dsl::auto_type]
fn report_is_not_resolved() -> _ {
  post_report::resolved
    .or(proposal_report::resolved)
    .or(category_report::resolved)
    .is_distinct_from(true)
}

impl InternalToCombinedView for ReportCombinedViewInternal {
  type CombinedView = ReportCombinedView;

  fn map_to_enum(self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self;

    if let (Some(post_report), Some(post), Some(category), Some(post_creator)) = (
      v.post_report,
      v.post.clone(),
      v.category.clone(),
      v.item_creator.clone(),
    ) {
      Some(ReportCombinedView::Post(PostReportView {
        post_report,
        post,
        category: Some(category),
        post_creator,
        creator: v.report_creator,
        category_actions: v.category_actions,
        post_actions: v.post_actions,
        person_actions: v.person_actions,
        creator_is_admin: v.item_creator_is_admin,
      }))
    } else if let (
      Some(proposal_report),
      Some(proposal),
      Some(post),
      Some(category),
      Some(comment_creator),
    ) = (
      v.proposal_report,
      v.proposal,
      v.post,
      v.category.clone(),
      v.item_creator.clone(),
    ) {
      Some(ReportCombinedView::Proposal(ProposalReportView {
        proposal_report,
        proposal,
        post,
        category: Some(category),
        creator: v.report_creator,
        comment_creator,
        category_actions: v.category_actions,
        proposal_actions: v.proposal_actions,
        person_actions: v.person_actions,
        creator_is_admin: v.item_creator_is_admin,
      }))
    } else if let (Some(category), Some(category_report)) = (v.category, v.category_report) {
      Some(ReportCombinedView::Category(CategoryReportView {
        category_report,
        category: Some(category),
        creator: v.report_creator,
      }))
    } else {
      None
    }
  }
}
