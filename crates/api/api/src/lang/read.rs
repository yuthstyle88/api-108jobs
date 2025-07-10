use actix_web::{web, HttpResponse};
use lemmy_multilang::loader::Lang;
use lemmy_multilang::namespace::AllTranslations;
use lemmy_utils::error::{FastJobError, FastJobErrorType, FastJobResult};
use std::fs;

pub async fn get_lang(path: web::Path<(String, String)>) -> FastJobResult<HttpResponse> {
  let (lang, filename) = path.into_inner();

  let allowed_languages = ["en", "th", "vi"];
  if !allowed_languages.contains(&lang.as_str()) {
    return Err(FastJobError::from(FastJobErrorType::InvalidUrl));
  }

  // Get the absolute path
  let base_path = std::env::current_dir()?.join("../../../../multilang/translations/website");
  let file_path = base_path.join(lang).join(filename);

  match fs::read_to_string(file_path) {
    Ok(content) => Ok(
      HttpResponse::Ok()
        .content_type("application/json")
        .body(content),
    ),
    Err(_) => Err(FastJobError::from(FastJobErrorType::FileNotFound)),
  }
}

pub async fn get_namespace(
  path: web::Path<(String, String)>,
  data: web::Data<AllTranslations>,
) -> FastJobResult<HttpResponse> {
  let (lang_str, filename) = path.into_inner();

  if let Some(lang) = Lang::from_str(&lang_str) {
    if let Some(ns_map) = data.get(&lang).and_then(|map| map.get(&filename)) {
      return Ok(HttpResponse::Ok().json(&ns_map.0));
    }
  }

  Err(FastJobError::from(FastJobErrorType::InvalidUrl))
}
