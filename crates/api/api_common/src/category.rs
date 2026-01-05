pub use app_108jobs_db_schema::{
    newtypes::{CategoryId, TagId},
    source::{
    category::{Category, CategoryActions},
    tag::{Tag, TagsView},
  },
};
pub use app_108jobs_db_schema_file::enums::CategoryVisibility;
pub use app_108jobs_db_views_category::{
  api::{
    CategoryResponse,
    GetCategory,
    GetCategoryResponse,
    GetRandomCategory,
    ListCategories,
    ListCategoriesResponse,
  },
  CategoryView,
};

pub mod actions {
  pub use app_108jobs_db_views_category::api::{
    CreateCategory,
    HideCategory,
  };

  pub mod moderation {
    pub use app_108jobs_db_schema_file::enums::CategoryFollowerState;
    pub use app_108jobs_db_views_category::api::{
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
