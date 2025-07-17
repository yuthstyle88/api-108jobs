use actix_web::{web, HttpResponse};
use convert_case::{Case, Casing};
use serde_json::{Map, Value};
use lemmy_multilang::loader::Lang;
use lemmy_multilang::namespace::AllTranslations;
use lemmy_utils::error::{FastJobError, FastJobErrorType, FastJobResult};

pub async fn get_namespace(
  path: web::Path<(String, String)>,
  data: web::Data<AllTranslations>,
) -> FastJobResult<HttpResponse> {
  let (lang_str, filename) = path.into_inner();

  if let Some(lang) = Lang::from_str(&lang_str) {
    if let Some(ns_map) = data.get(&lang).and_then(|map| map.get(&filename)) {
      let camel_cased_map: Map<String, Value> = ns_map.0.iter()
          .map(|(k, v)| (k.to_case(Case::Camel), Value::String(v.clone())))
          .collect();

      return Ok(HttpResponse::Ok().json(camel_cased_map));
    }
  }

  Err(FastJobError::from(FastJobErrorType::InvalidUrl))
}
