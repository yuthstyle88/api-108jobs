use actix_web::{web::{Data, Path}, HttpResponse};
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use lemmy_api_utils::context::FastJobContext;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use crate::utils::user_files_dir;

pub async fn get_file(
  path: Path<(i32, String)>,
  _context: Data<FastJobContext>,
) -> FastJobResult<HttpResponse> {
  let (user_id, filename) = path.into_inner();
  let file_path = user_files_dir(user_id).join(&filename);

  if !file_path.exists() {
    return Err(FastJobErrorType::FileNotFound.into());
  }

  let bytes = tokio::fs::read(&file_path).await?;

  // Build safe Content-Disposition header
  let cd = ContentDisposition {
    disposition: DispositionType::Attachment,
    parameters: vec![DispositionParam::Filename(filename.clone().into())],
  };

  Ok(HttpResponse::Ok()
      .insert_header(cd)
      .content_type("application/octet-stream")
      .body(bytes))
}
