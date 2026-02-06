use app_108jobs_db_schema::{
    newtypes::{CategoryId, Coin, PaginationCursor, PersonId},
    source::{
    combined::search::SearchCombined,
    comment::{Comment, CommentActions},
    category::{Category, CategoryActions},
    images::ImageDetails,
    instance::InstanceActions,
    delivery_details::DeliveryDetails,
    person::{Person, PersonActions},
    post::{Post, PostActions},
    tag::TagsView,
  },
    SearchSortType,
    SearchType,
};
use app_108jobs_db_schema_file::enums::{IntendedUse, JobType, ListingType, PostKind};
use app_108jobs_db_views_comment::CommentView;
use app_108jobs_db_views_category::CategoryView;
use app_108jobs_db_views_person::PersonView;
use app_108jobs_db_views_post::PostView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  app_108jobs_db_schema::utils::queries::{
    category_post_tags_fragment,
    creator_banned,
    creator_is_admin,
    local_user_can_mod,
    post_tags_fragment,
  },
  app_108jobs_db_schema::utils::queries::{creator_banned_from_category, creator_is_moderator},
  app_108jobs_db_views_local_user::LocalUserView,
};
use app_108jobs_db_schema::newtypes::LanguageId;

#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined search view
pub(crate) struct SearchCombinedViewInternal {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub search_combined: SearchCombined,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment: Option<Comment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Option<Post>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub item_creator: Option<Person>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category: Option<Category>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category_actions: Option<CategoryActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post_actions: Option<PostActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment_actions: Option<CommentActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub image_details: Option<ImageDetails>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub delivery_details: Option<DeliveryDetails>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_is_admin()
    )
  )]
  pub item_creator_is_admin: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_tags_fragment()
    )
  )]
  /// tags of this post
  pub post_tags: TagsView,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = category_post_tags_fragment()
    )
  )]
  /// available tags in this category
  pub category_post_tags: TagsView,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user_can_mod()
    )
  )]
  pub can_mod: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_banned()
    )
  )]
  pub creator_banned: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_is_moderator()
    )
  )]
  pub creator_is_moderator: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_banned_from_category()
    )
  )]
  pub creator_banned_from_category: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
// Use serde's internal tagging, to work easier with javascript libraries
#[serde(tag = "type_")]
pub enum SearchCombinedView {
  Post(PostView),
  Comment(CommentView),
  Category(CategoryView),
  Person(PersonView),
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Searches the site, given a search term, and some optional filters.
#[serde(rename_all = "camelCase")]
pub struct Search {
  pub q: Option<String>,
  pub category_id: Option<CategoryId>,
  pub language_id: Option<LanguageId>,
  pub category_name: Option<String>,
  pub creator_id: Option<PersonId>,
  pub r#type: Option<SearchType>,
  pub sort: Option<SearchSortType>,
  /// Filter to within a given time range, in seconds.
  /// IE 60 would give results for the past minute.
  pub time_range_seconds: Option<i32>,
  pub listing_type: Option<ListingType>,
  pub title_only: Option<bool>,
  pub post_url_only: Option<bool>,
  pub liked_only: Option<bool>,
  pub disliked_only: Option<bool>,
  /// If true, then show the self_promotion posts (even if your user setting is to hide them)
  pub self_promotion: Option<bool>,
  pub intended_use: Option<IntendedUse>,
  pub job_type: Option<JobType>,
  /// Minimum budget in cents (Coin type)
  pub budget_min: Option<Coin>,
  /// Maximum budget in cents (Coin type)
  pub budget_max: Option<Coin>,
  pub requires_english: Option<bool>,
  pub post_kind: Option<PostKind>,
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The search response, containing lists of the return type possibilities
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
  pub results: Vec<SearchCombinedView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}
