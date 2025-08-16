pub use lemmy_db_views_person_liked_combined::{
  ListPersonLiked,
  ListPersonLikedResponse,
  PersonLikedCombinedView,
};
pub use lemmy_db_views_person_saved_combined::{
  ListPersonSaved,
  ListPersonSavedResponse,
  PersonSavedCombinedView,
};
pub use lemmy_db_views_site::api::{
  DeleteAccount,
  ListPersonHidden,
  ListPersonHiddenResponse,
  ListPersonRead,
  ListPersonReadResponse,
  MyUserInfo,
  SaveUserSettings,
};

pub mod auth {
  pub use lemmy_db_schema::source::login_token::LoginToken;
  pub use lemmy_db_views_registration_applications::api::Register;
  pub use lemmy_db_views_site::api::{
    CaptchaResponse,
    ChangePassword,
    GenerateTotpSecretResponse,
    GetCaptchaResponse,
    ListLoginsResponse,
    Login,
    LoginResponse,
    PasswordChangeAfterReset,
    PasswordReset,
    ResendVerificationEmail,
    UpdateTotp,
    UpdateTotpResponse,
    VerifyEmail,
  };
}

use serde::{Deserialize, Serialize};

// Delete request structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteItemRequest<T> {
    pub id: T,
}
