use serde::Deserialize;
use lemmy_db_schema::newtypes::OAuthProviderId;
use lemmy_db_schema::sensitive::SensitiveString;
use lemmy_utils::error::{FastJobError, FastJobErrorType};
use crate::api::Login;

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub username_or_email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OAuthUserLoginRequest {
    pub user_id: String,
    pub name: String,
    pub email: String,
    pub access_token: String,
    pub provider_id: OAuthProviderId,
}

impl TryFrom<LoginRequest> for Login {
    type Error = FastJobError;

    fn try_from(login: LoginRequest) -> Result<Self, Self::Error> {
        if login.username_or_email.trim().is_empty() {
            return Err(FastJobError::from(FastJobErrorType::EmptyUsername));
        }

        if login.password.trim().is_empty() {
            return Err(FastJobError::from(FastJobErrorType::EmptyPassword));
        }

        if login.password.len() < 6 || login.password.len() > 32 {
            return Err(FastJobError::from(FastJobErrorType::InvalidPasswordLength));
        }

        Ok(Self {
            username_or_email: SensitiveString::from(login.username_or_email),
            password: SensitiveString::from(login.password),
            totp_2fa_token: None,
        })
    }
}