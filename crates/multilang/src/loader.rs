use maplit::hashmap;
use once_cell::sync::Lazy;
use std::{collections::HashMap, fs, path::Path};
use strfmt::strfmt;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Lang {
  Th,
  Vi,
  En,
}

impl Lang {
  pub fn all() -> &'static [Lang] {
    &[Lang::En, Lang::Vi, Lang::Th]
  }

  pub fn as_str(&self) -> &'static str {
    match self {
      Lang::En => "en",
      Lang::Vi => "vi",
      Lang::Th => "th",
    }
  }

  pub fn from_str(s: &str) -> Option<Self> {
    match s {
      "en" => Some(Lang::En),
      "vi" => Some(Lang::Vi),
      "th" => Some(Lang::Th),
      _ => None,
    }
  }

  pub fn default() -> Self {
    Lang::En
  }

  pub fn verify_email_subject(&self, hostname: impl ToString) -> String {
    let bundle = ALL_BUNDLES.get(self).unwrap();
    let template = bundle.get("verify_email_subject").unwrap();

    strfmt(
      template,
      &hashmap! {
          "hostname".to_string() => hostname.to_string()
      },
    )
    .expect("Missing placeholder in template")
  }

  pub fn password_reset_subject(&self, username: impl ToString) -> String {
    let bundle = ALL_BUNDLES
      .get(self)
      .unwrap_or_else(|| panic!("Missing language bundle for {:?}", self));

    let template = bundle
      .get("password_reset_subject")
      .unwrap_or_else(|| panic!("Missing key 'password_reset_subject' for {:?}", self));

    let vars = HashMap::from([("username".to_string(), username.to_string())]);

    strfmt(template, &vars).expect("Interpolation error in password_reset_subject")
  }

  pub fn password_reset_body(&self, username: impl ToString, reset_link: impl ToString) -> String {
    let bundle = ALL_BUNDLES
      .get(self)
      .unwrap_or_else(|| panic!("Missing language bundle for {:?}", self));

    let template = bundle
      .get("password_reset_body")
      .unwrap_or_else(|| panic!("Missing key 'password_reset_body' for {:?}", self));

    let vars = HashMap::from([
      ("username".to_string(), username.to_string()),
      ("reset_link".to_string(), reset_link.to_string()),
    ]);

    strfmt(template, &vars).expect("Interpolation error in password_reset_body")
  }

  pub fn email_verified_subject(&self, username: impl ToString) -> String {
    let bundle = ALL_BUNDLES
      .get(self)
      .unwrap_or_else(|| panic!("Missing language bundle for {:?}", self));

    let template = bundle
      .get("email_verified_subject")
      .unwrap_or_else(|| panic!("Missing key 'email_verified_subject' for {:?}", self));

    let vars = HashMap::from([("username".to_string(), username.to_string())]);

    strfmt(template, &vars).expect("Interpolation error in email_verified_subject")
  }

  pub fn email_verified_body(&self) -> String {
    let bundle = ALL_BUNDLES
      .get(self)
      .unwrap_or_else(|| panic!("Missing language bundle for {:?}", self));

    bundle
      .get("email_verified_body")
      .unwrap_or_else(|| panic!("Missing key 'email_verified_body' for {:?}", self))
      .to_string()
  }
}

type Bundle = HashMap<String, String>;

const LANGUAGE_PATHS: &[(Lang, &str)] = &[
  (Lang::En, "crates/multilang/translations/email/en.json"),
  (Lang::Vi, "crates/multilang/translations/email/vi.json"),
  (Lang::Th, "crates/multilang/translations/email/th.json"),
];

pub static ALL_BUNDLES: Lazy<HashMap<Lang, Bundle>> = Lazy::new(|| {
  LANGUAGE_PATHS
    .iter()
    .map(|(lang, path)| {
      let json = fs::read_to_string(Path::new(path))
        .unwrap_or_else(|e| panic!("Failed to read translation file at '{}': {e}", path));

      let parsed: Bundle = serde_json::from_str(&json)
        .unwrap_or_else(|e| panic!("Invalid JSON in file '{}': {e}", path));

      (*lang, parsed)
    })
    .collect()
});
