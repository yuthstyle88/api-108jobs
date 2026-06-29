use app_108jobs_db::{
  newtypes::{PaginationCursor, PersonId},
  source::{
    category::{Category, CategoryActions},
    combined::person_content::PersonContentCombined,
    delivery_details::DeliveryDetails,
    images::ImageDetails,
    instance::InstanceActions,
    person::{Person, PersonActions},
    post::{Post, PostActions},
    proposal::{Proposal, ProposalActions},
    tag::TagsView,
  },
  PersonContentType,
};
use app_108jobs_db_views_post::PostView;
use app_108jobs_db_views_proposal::ProposalView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  app_108jobs_db::utils::queries::{
    creator_banned, creator_is_admin, local_user_can_mod, post_tags_fragment,
  },
  app_108jobs_db::utils::queries::{creator_banned_from_category, creator_is_moderator},
  app_108jobs_db_views_local_user::LocalUserView,
  diesel::{Queryable, Selectable},
};

#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined person_content view
pub(crate) struct PersonContentCombinedViewInternal {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_content_combined: PersonContentCombined,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub proposal: Option<Proposal>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Post,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub item_creator: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category: Category,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub category_actions: Option<CategoryActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post_actions: Option<PostActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub proposal_actions: Option<ProposalActions>,
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
  pub post_tags: TagsView,
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
pub enum PersonContentCombinedView {
  Post(PostView),
  Proposal(ProposalView),
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets a person's content (posts and comments)
///
/// Either person_id, or username are required.
pub struct ListPersonContent {
  pub type_: Option<PersonContentType>,
  pub person_id: Option<PersonId>,
  /// Example: dessalines , or dessalines@xyz.tld
  pub username: Option<String>,
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A person's content response.
#[serde(rename_all = "camelCase")]
pub struct ListPersonContentResponse {
  pub content: Vec<PersonContentCombinedView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}
