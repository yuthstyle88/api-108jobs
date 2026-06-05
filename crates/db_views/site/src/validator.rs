//! Validation logic for site requests
use crate::api::{AuthenticateWithOauth, AuthenticateWithOauthRequest};
use app_108jobs_db_schema::newtypes::OAuthProviderId;
use app_108jobs_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

/// Validates that OAuth code is not empty
pub fn validate_oauth_code(code: &str) -> FastJobResult<()> {
  if code.trim().is_empty() {
    return Err(FastJobErrorType::InvalidField("OAuth code cannot be empty".to_string()).into());
  }
  Ok(())
}

/// Validates that OAuth provider ID is positive
pub fn validate_oauth_provider_id(provider_id: OAuthProviderId) -> FastJobResult<()> {
  if provider_id.0 <= 0 {
    return Err(FastJobErrorType::InvalidArgument.into());
  }
  Ok(())
}

// ============================================================================
// Validated Request Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct ValidAuthenticateWithOauthRequest(pub AuthenticateWithOauthRequest);

impl TryFrom<AuthenticateWithOauthRequest> for ValidAuthenticateWithOauthRequest {
  type Error = FastJobError;

  fn try_from(value: AuthenticateWithOauthRequest) -> Result<Self, Self::Error> {
    validate_oauth_code(&value.code)?;
    validate_oauth_provider_id(value.oauth_provider_id)?;

    Ok(ValidAuthenticateWithOauthRequest(value))
  }
}

#[derive(Debug, Clone)]
pub struct ValidAuthenticateWithOauth(pub AuthenticateWithOauth);

impl TryFrom<AuthenticateWithOauthRequest> for ValidAuthenticateWithOauth {
  type Error = FastJobError;

  fn try_from(value: AuthenticateWithOauthRequest) -> Result<Self, Self::Error> {
    validate_oauth_code(&value.code)?;
    validate_oauth_provider_id(value.oauth_provider_id)?;

    Ok(ValidAuthenticateWithOauth(AuthenticateWithOauth {
      code: value.code,
      oauth_provider_id: value.oauth_provider_id,
      redirect_uri: value.redirect_uri,
      self_promotion: Some(false),
      username: value.email.map(|e| e.into()),
      name: value.name,
      answer: value.answer,
      pkce_code_verifier: None,
    }))
  }
}
