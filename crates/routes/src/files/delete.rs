use crate::{
  files::DeleteFileRequest,
  utils::{sanitize_filename, user_files_dir},
};
use actix_web::web::{Data, Json, Path};
use app_108jobs_api_utils::context::FastJobContext;
use app_108jobs_core::error::{FastJobErrorType, FastJobResult};
use app_108jobs_db_views_local_user::LocalUserView;
use app_108jobs_db_views_site::api::SuccessResponse;
use tokio::fs;

pub async fn delete_file(
  path: Path<DeleteFileRequest>,
  local_user_view: LocalUserView,
  _context: Data<FastJobContext>,
) -> FastJobResult<Json<SuccessResponse>> {
  let filename = sanitize_filename(&path.filename);
  if filename.is_empty() {
    return Err(FastJobErrorType::InvalidBodyField.into());
  }

  let dir = user_files_dir(local_user_view.local_user.id.0);
  let target = dir.join(&filename);

  if !target.exists() {
    return Err(FastJobErrorType::FileNotFound.into());
  }

  match fs::remove_file(&target).await {
    Ok(_) => Ok(Json(SuccessResponse { success: true })),
    Err(_) => Err(FastJobErrorType::CouldntDeleteFile.into()),
  }
}
