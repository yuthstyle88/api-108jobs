//! Validation logic for category requests
use crate::api::{CreateCategory, CreateCategoryRequest};
use app_108jobs_db_schema::newtypes::{CategoryId, TagId};
use app_108jobs_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

/// Validates that category name is not empty
pub fn validate_category_name(name: &str) -> FastJobResult<()> {
  if name.trim().is_empty() {
    return Err(FastJobErrorType::InvalidField("category name cannot be empty".to_string()).into());
  }
  Ok(())
}

/// Validates that category title is not empty
pub fn validate_category_title(title: &str) -> FastJobResult<()> {
  if title.trim().is_empty() {
    return Err(FastJobErrorType::InvalidField("category title cannot be empty".to_string()).into());
  }
  Ok(())
}

/// Validates that tag display name is not empty
pub fn validate_tag_display_name(display_name: &str) -> FastJobResult<()> {
  if display_name.trim().is_empty() {
    return Err(FastJobErrorType::InvalidField("tag display name cannot be empty".to_string()).into());
  }
  Ok(())
}

/// Validates that category ID is positive
pub fn validate_category_id(category_id: CategoryId) -> FastJobResult<()> {
  if category_id.0 <= 0 {
    return Err(FastJobErrorType::InvalidArgument.into());
  }
  Ok(())
}

/// Validates that tag ID is positive
pub fn validate_tag_id(tag_id: TagId) -> FastJobResult<()> {
  if tag_id.0 <= 0 {
    return Err(FastJobErrorType::InvalidArgument.into());
  }
  Ok(())
}

// ============================================================================
// Validated Request Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct ValidCreateCategoryRequest(pub CreateCategoryRequest);

impl TryFrom<CreateCategoryRequest> for ValidCreateCategoryRequest {
  type Error = FastJobError;

  fn try_from(value: CreateCategoryRequest) -> Result<Self, Self::Error> {
    // Name and title are optional in CreateCategoryRequest, but should be validated if provided
    if let Some(ref name) = value.name {
      validate_category_name(name)?;
    }
    if let Some(ref title) = value.title {
      validate_category_title(title)?;
    }

    // Validate parent_id if provided
    if let Some(parent_id) = value.parent_id {
      validate_category_id(parent_id)?;
    }

    Ok(ValidCreateCategoryRequest(value))
  }
}

#[derive(Debug, Clone)]
pub struct ValidCreateCategory(pub CreateCategory);

impl TryFrom<CreateCategoryRequest> for ValidCreateCategory {
  type Error = FastJobError;

  fn try_from(value: CreateCategoryRequest) -> Result<Self, Self::Error> {
    // For CreateCategory, name and title are required
    let name = value.name.ok_or_else(|| {
      FastJobErrorType::InvalidField("category name is required".to_string())
    })?;
    let title = value.title.ok_or_else(|| {
      FastJobErrorType::InvalidField("category title is required".to_string())
    })?;

    validate_category_name(&name)?;
    validate_category_title(&title)?;

    // Validate parent_id if provided
    if let Some(parent_id) = value.parent_id {
      validate_category_id(parent_id)?;
    }

    Ok(ValidCreateCategory(CreateCategory {
      name,
      title,
      sidebar: None,
      description: value.description,
      icon: value.icon,
      banner: value.banner,
      self_promotion: value.self_promotion,
      posting_restricted_to_mods: None,
      discussion_languages: None,
      visibility: None,
      is_new: value.is_new,
      parent_id: value.parent_id,
    }))
  }
}
