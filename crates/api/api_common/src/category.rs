pub use lemmy_db_schema::{
    newtypes::{CategoryId, TagId},
    source::{
    category::{Category, CategoryActions},
    tag::{Tag, TagsView},
  },
};
pub use lemmy_db_schema_file::enums::CategoryVisibility;
pub use lemmy_db_views_category::{
  api::{
    CategoryResponse,
    GetCategory,
    GetCategoryResponse,
    GetRandomCategory,
    ListCommunities,
    ListCommunitiesResponse,
  },
  CategoryView,
};

pub mod actions {
  pub use lemmy_db_views_category::api::{
    CreateCategory,
    HideCategory,
  };

  pub mod moderation {
    pub use lemmy_db_schema_file::enums::CategoryFollowerState;
    pub use lemmy_db_views_category::api::{
      CategoryIdQuery,
      CreateCategoryTag,
      DeleteCategory,
      DeleteCategoryTag,
      EditCategory,
      PurgeCategory,
      RemoveCategory,
      UpdateCategoryTag,
    };
  }
}
