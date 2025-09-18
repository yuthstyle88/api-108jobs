use actix_web::{web::{Data, Path}, HttpResponse};
use lemmy_api_utils::context::FastJobContext;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use std::path::PathBuf;

fn user_files_dir(user_id: i32) -> PathBuf {
  PathBuf::from("uploads").join("files").join(user_id.to_string())
}

#[derive(Debug)]
pub struct FilePath {
  pub user_id: i32,
  pub filename: String,
}

pub async fn get_file(path: Path<(i32, String)>, _context: Data<FastJobContext>) -> FastJobResult<HttpResponse> {
  let (user_id, filename) = path.into_inner();
  let file_path = user_files_dir(user_id).join(&filename);

  if !file_path.exists() {
    return Err(FastJobErrorType::FileNotFound.into());
  }

  let bytes = tokio::fs::read(&file_path).await?;

  Ok(
    HttpResponse::Ok()
      .append_header((
        "Content-Disposition",
        format!("attachment; filename=\"{}\"", filename),
      ))
      .content_type("application/octet-stream")
      .body(bytes),
  )
}
