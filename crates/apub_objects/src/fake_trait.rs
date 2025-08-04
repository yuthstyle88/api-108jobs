use std::fmt::Debug;
use actix_web::web::Data;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

pub trait Object: Sized + Debug {
  /// App data type passed to handlers. Must be identical to
  /// [crate::config::FederationConfigBuilder::app_data] type.
  type DataType: Clone + Send + Sync;
  /// The type of protocol struct which gets sent over network to federate this database struct.
  type Kind;
  /// Error type returned by handler methods
  type Error;

  /// `id` field of the object
  fn id(&self) -> &Url;
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PublicKey(Url,String);

pub struct Note;
impl PublicKey {
  fn new(id: Url, pem: String) -> Self {
    Self(id, pem)
  }
}
pub trait Actor: Object + Send + 'static {
  /// The actor's public key for verifying signatures of incoming activities.
  ///
  /// Use [generate_actor_keypair](crate::http_signatures::generate_actor_keypair) to create the
  /// actor keypair.
  fn public_key_pem(&self) -> &str;

  /// The actor's private key for signing outgoing activities.
  ///
  /// Use [generate_actor_keypair](crate::http_signatures::generate_actor_keypair) to create the
  /// actor keypair.
  fn private_key_pem(&self) -> Option<String>;

  /// The inbox where activities for this user should be sent to
  fn inbox(&self) -> Url;

  /// Generates a public key struct for use in the actor json representation
  fn public_key(&self) -> PublicKey {
    PublicKey::new(self.id().clone(), self.public_key_pem().to_string())
  }

  /// The actor's shared inbox, if any
  fn shared_inbox(&self) -> Option<Url> {
    None
  }

  /// Returns shared inbox if it exists, normal inbox otherwise.
  fn shared_inbox_or_inbox(&self) -> Url {
    self.shared_inbox().unwrap_or_else(|| self.inbox())
  }
}