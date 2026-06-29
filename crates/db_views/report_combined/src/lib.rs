use app_108jobs_db::source::{
  category::{Category, CategoryActions},
  category_report::CategoryReport,
  combined::report::ReportCombined,
  person::{Person, PersonActions},
  post::{Post, PostActions},
  post_report::PostReport,
  proposal::{Proposal, ProposalActions},
  proposal_report::ProposalReport,
};
use app_108jobs_db_views_reports::{CategoryReportView, PostReportView, ProposalReportView};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {
  app_108jobs_db::{
    utils::queries::{local_user_is_admin, person1_select},
    Person1AliasAllColumnsTuple,
  },
  app_108jobs_db_views_local_user::LocalUserView,
  diesel::{dsl::Nullable, NullableExpressionMethods, Queryable, Selectable},
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined report view
#[serde(rename_all = "camelCase")]
pub struct ReportCombinedViewInternal {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub report_combined: ReportCombined,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post_report: Option<PostReport>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub proposal_report: Option<ProposalReport>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category_report: Option<CategoryReport>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub report_creator: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub proposal: Option<Proposal>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Option<Post>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<Person1AliasAllColumnsTuple>,
      select_expression = person1_select().nullable()
    )
  )]
  pub item_creator: Option<Person>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category: Option<Category>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category_actions: Option<CategoryActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post_actions: Option<PostActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub proposal_actions: Option<ProposalActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user_is_admin()
    )
  )]
  pub item_creator_is_admin: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
// Use serde's internal tagging, to work easier with javascript libraries
#[serde(tag = "type_")]
pub enum ReportCombinedView {
  Post(PostReportView),
  Proposal(ProposalReportView),
  Category(CategoryReportView),
}
