use actix_web::{web, HttpResponse};
use std::fs;
use lemmy_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

pub async fn get_lang(path: web::Path<(String, String)>) -> FastJobResult<HttpResponse> {
    let (lang, filename) = path.into_inner();

    let allowed_languages = ["en", "th", "vi"];
    if !allowed_languages.contains(&lang.as_str()) {
        return Err(FastJobError::from(FastJobErrorType::InvalidUrl));
    }

    // Get the absolute path
    let base_path = std::env::current_dir()?.join("crates/multilang/translations/translations");
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