use crate::{
  newtypes::OAuthProviderId,
};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::oauth_provider;
use serde::{
  ser::{SerializeStruct, Serializer},
  Deserialize,
  Serialize,
};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = oauth_provider))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// oauth provider with client_secret - should never be sent to the client
pub struct OAuthProvider {
  pub id: OAuthProviderId,
  /// The OAuth 2.0 provider name displayed to the user on the Login page
  pub display_name: String,
  /// Automatically sets multilang as verified on registration
  pub auto_verify_email: bool,
  /// Allows linking an OAUTH account to an existing user account by matching emails
  pub account_linking_enabled: bool,
  /// switch to enable or disable an oauth provider
  pub enabled: bool,
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
// A subset of OAuthProvider used for public requests, for example to display the OAUTH buttons on
// the login page
pub struct PublicOAuthProvider(pub OAuthProvider);

impl Serialize for PublicOAuthProvider {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut state = serializer.serialize_struct("PublicOAuthProvider", 5)?;
    state.serialize_field("id", &self.0.id)?;
    state.serialize_field("display_name", &self.0.display_name)?;
    state.end()
  }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = oauth_provider))]
pub struct OAuthProviderInsertForm {
  pub display_name: String,
  pub auto_verify_email: Option<bool>,
  pub account_linking_enabled: Option<bool>,
  pub enabled: Option<bool>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = oauth_provider))]
pub struct OAuthProviderUpdateForm {
  pub display_name: Option<String>,
  pub auto_verify_email: Option<bool>,
  pub account_linking_enabled: Option<bool>,
  pub enabled: Option<bool>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
}
