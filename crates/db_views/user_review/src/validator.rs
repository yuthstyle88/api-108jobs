//! Validation logic for user review requests
use crate::api::{SubmitUserReviewForm, SubmitUserReviewRequest};
use app_108jobs_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

/// Validates that a rating is between 1 and 5 (inclusive)
pub fn validate_rating(rating: i16) -> FastJobResult<()> {
  if rating < 1 || rating > 5 {
    return Err(FastJobErrorType::InvalidArgument.into());
  }
  Ok(())
}

// ============================================================================
// Validated Request Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct ValidSubmitUserReviewRequest(pub SubmitUserReviewRequest);

impl TryFrom<SubmitUserReviewRequest> for ValidSubmitUserReviewRequest {
  type Error = FastJobError;

  fn try_from(value: SubmitUserReviewRequest) -> Result<Self, Self::Error> {
    validate_rating(value.rating)?;
    Ok(ValidSubmitUserReviewRequest(value))
  }
}

impl TryFrom<ValidSubmitUserReviewRequest> for SubmitUserReviewForm {
  type Error = FastJobError;

  fn try_from(value: ValidSubmitUserReviewRequest) -> Result<Self, Self::Error> {
    Ok(SubmitUserReviewForm {
      reviewee_id: value.0.reviewee_id,
      workflow_id: value.0.workflow_id,
      rating: value.0.rating,
      comment: value.0.comment,
    })
  }
}
