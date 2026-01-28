use app_108jobs_db_schema::source::{
  comment::{Comment, CommentActions},
  comment_report::CommentReport,
  category::{Category, CategoryActions},
  category_report::CategoryReport,
  person::{Person, PersonActions},
  post::{Post, PostActions},
  post_report::PostReport,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  app_108jobs_db_schema::{
    utils::queries::{
      comment_creator_is_admin,
      person1_select,
      post_creator_is_admin,
    },
    Person1AliasAllColumnsTuple,
  },
};

pub mod api;
#[cfg(feature = "full")]
pub mod comment_report_view;

#[cfg(feature = "full")]
pub mod category_report_view;

#[cfg(feature = "full")]
pub mod post_report_view;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A comment report view.
#[serde(rename_all = "camelCase")]
pub struct CommentReportView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment_report: CommentReport,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment: Comment,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Post,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category: Option<Category>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Person1AliasAllColumnsTuple,
      select_expression = person1_select()
    )
  )]
  pub comment_creator: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment_actions: Option<CommentActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category_actions: Option<CategoryActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = comment_creator_is_admin()
    )
  )]
  pub creator_is_admin: bool,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A category report view.
#[serde(rename_all = "camelCase")]
pub struct CategoryReportView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category_report: CategoryReport,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category: Option<Category>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A post report view.
#[serde(rename_all = "camelCase")]
pub struct PostReportView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post_report: PostReport,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Post,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category: Option<Category>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Person1AliasAllColumnsTuple,
      select_expression = person1_select()
    )
  )]
  pub post_creator: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category_actions: Option<CategoryActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post_actions: Option<PostActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_creator_is_admin()
    )
  )]
  pub creator_is_admin: bool,
}
