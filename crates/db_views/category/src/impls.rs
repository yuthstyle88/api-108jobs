use crate::api::{CreateCategory, CreateCategoryGroup, CreateCategoryGroupRequest, CreateCategoryRequest, CreateSubCategory, CreateSubCategoryRequest};
use crate::CategoryView;
use lemmy_utils::error::{FastJobError, FastJobErrorType};
use validator::Validate;

impl CategoryView {}

/// validate category group request
impl TryFrom<CreateCategoryGroupRequest> for CreateCategoryGroup {
  type Error = FastJobError;

  fn try_from(mut value: CreateCategoryGroupRequest) -> Result<Self, Self::Error> {
    value
        .validate()
        .map_err(|err| FastJobErrorType::ValidationError(err.to_string()))?;

    Ok(CreateCategoryGroup {
      title: value.title.take().unwrap(),
      sort_order: value.sort_order.take().unwrap(),
    })
  }
}

/// validate category request
impl TryFrom<CreateCategoryRequest> for CreateCategory {
  type Error = FastJobError;

  fn try_from(mut value: CreateCategoryRequest) -> Result<Self, Self::Error> {
    value
      .validate()
      .map_err(|err| FastJobErrorType::ValidationError(err.to_string()))?;

    Ok(CreateCategory {
      title: value.title.take().unwrap(),
      slug: value.slug.take(),
      image: value.image.take(),
      is_new: value.is_new,
      sort_order: value.sort_order,
    })
  }
}

/// validate subcategory request
impl TryFrom<CreateSubCategoryRequest> for CreateSubCategory {
  type Error = FastJobError;

  fn try_from(mut value: CreateSubCategoryRequest) -> Result<Self, Self::Error> {
    value
        .validate()
        .map_err(|err| FastJobErrorType::ValidationError(err.to_string()))?;

    Ok(CreateSubCategory {
      title: value.title.take().unwrap(),
      slug: value.slug.take(),
      image: value.image.take(),
      is_new: value.is_new,
      sort_order: value.sort_order.take().unwrap(),
      parent_id: value.parent_id.take().unwrap(),
      group_id: value.group_id.take().unwrap(),
    })
  }
}
