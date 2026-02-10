//! Validation logic for comment requests
use crate::api::{CreateComment, CreateCommentRequest};
use app_108jobs_db_schema::newtypes::{CommentId, LanguageId, PostId};
use app_108jobs_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

/// Validates that comment content is not empty
pub fn validate_comment_content(content: &str) -> FastJobResult<()> {
  if content.trim().is_empty() {
    return Err(FastJobErrorType::InvalidField("comment content cannot be empty".to_string()).into());
  }
  Ok(())
}

/// Validates that post ID is positive
pub fn validate_post_id(post_id: PostId) -> FastJobResult<()> {
  if post_id.0 <= 0 {
    return Err(FastJobErrorType::InvalidArgument.into());
  }
  Ok(())
}

/// Validates that language ID is positive
pub fn validate_language_id(language_id: LanguageId) -> FastJobResult<()> {
  if language_id.0 <= 0 {
    return Err(FastJobErrorType::InvalidArgument.into());
  }
  Ok(())
}

/// Validates that parent comment ID (if provided) is positive
pub fn validate_parent_comment_id(parent_id: Option<CommentId>) -> FastJobResult<()> {
  if let Some(id) = parent_id {
    if id.0 <= 0 {
      return Err(FastJobErrorType::InvalidArgument.into());
    }
  }
  Ok(())
}

// ============================================================================
// Validated Request Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct ValidCreateCommentRequest(pub CreateCommentRequest);

impl TryFrom<CreateCommentRequest> for ValidCreateCommentRequest {
  type Error = FastJobError;

  fn try_from(value: CreateCommentRequest) -> Result<Self, Self::Error> {
    validate_comment_content(&value.content)?;
    validate_post_id(value.post_id)?;
    validate_language_id(value.language_id)?;
    validate_parent_comment_id(value.parent_id)?;

    Ok(ValidCreateCommentRequest(value))
  }
}

#[derive(Debug, Clone)]
pub struct ValidCreateComment(pub CreateComment);

impl TryFrom<CreateCommentRequest> for ValidCreateComment {
  type Error = FastJobError;

  fn try_from(value: CreateCommentRequest) -> Result<Self, Self::Error> {
    validate_comment_content(&value.content)?;
    validate_post_id(value.post_id)?;
    validate_parent_comment_id(value.parent_id)?;

    Ok(ValidCreateComment(CreateComment {
      content: value.content,
      post_id: value.post_id,
      parent_id: value.parent_id,
      language_id: Some(value.language_id),
    }))
  }
}
