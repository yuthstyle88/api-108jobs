// Avoid warnings for unused 0.19 website
#![allow(dead_code)]

use crate::loader::Lang;
use crate::namespace::{AllTranslations, NamespaceTranslations};
use lemmy_db_schema::sensitive::SensitiveString;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub mod loader;
pub mod namespace;

#[allow(clippy::expect_used)]
fn user_language(local_user_view: &LocalUserView) -> Lang {
  let preferred_lang = &local_user_view.local_user.interface_language;

  Lang::from_str(&preferred_lang).unwrap_or_else(|| {
    tracing::warn!(
      "Unsupported language '{}', falling back to default",
      preferred_lang
    );
    Lang::default()
  })
}

fn user_email(local_user_view: &LocalUserView) -> FastJobResult<SensitiveString> {
  local_user_view
    .local_user
    .email
    .clone()
    .ok_or(FastJobErrorType::EmailRequired.into())
}

pub fn load_all_translations(dir: &Path) -> FastJobResult<AllTranslations> {
  let mut all_translations = HashMap::new();

  for &lang in Lang::all() {
    let lang_dir = dir.join(lang.as_str());
    let mut namespaces = HashMap::new();

    if !lang_dir.exists() {
      continue;
    }

    for entry in fs::read_dir(&lang_dir)? {
      let entry = entry?;
      let path = entry.path();

      if is_json_file(&path) {
        let namespace = path
          .file_stem()
          .and_then(|s| s.to_str())
          .ok_or_else(|| anyhow::anyhow!("Invalid file name in {:?}", path))?
          .to_string();

        let content = fs::read_to_string(&path)?;
        let parsed: HashMap<String, String> =
          serde_json::from_str::<HashMap<String, String>>(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse {:?}: {}", path, e))?
            .into_iter()
            .map(|(k, v)| (to_camel_case(&k), v))
            .collect();

        namespaces.insert(namespace, NamespaceTranslations(parsed));
      }
    }

    all_translations.insert(lang, namespaces);
  }

  Ok(all_translations)
}
fn to_camel_case(input: &str) -> String {
  let mut result = String::new();
  let mut capitalize_next = false;

  for (i, ch) in input.chars().enumerate() {
    if ch == '_' {
      capitalize_next = true;
    } else if capitalize_next {
      result.push(ch.to_ascii_uppercase());
      capitalize_next = false;
    } else if i == 0 {
      result.push(ch.to_ascii_lowercase());
    } else {
      result.push(ch);
    }
  }

  result
}

fn is_json_file(path: &PathBuf) -> bool {
  path.extension().map_or(false, |ext| ext == "json")
}
