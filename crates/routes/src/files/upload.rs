use crate::files::{file_url, FileUploadResponse};
use crate::utils::{sanitize_filename, unique_target_filename, user_files_dir};
use actix_multipart::Multipart;
use actix_web::web::{Data, Json};
use futures_util::TryStreamExt as StreamExt;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use tokio::{fs, io::AsyncWriteExt};

const MAX_FILE_SIZE_BYTES: u64 = 25 * 1024 * 1024;

pub async fn upload_file(
  mut payload: Multipart,
  local_user_view: LocalUserView,
  context: Data<FastJobContext>,
) -> FastJobResult<Json<FileUploadResponse>> {
  // Only the first file field is considered
  let mut saved: Option<FileUploadResponse> = None;

  while let Some(item) = payload.try_next().await.map_err(|_| FastJobErrorType::InvalidBodyField)? {
    let content_disposition = item.content_disposition().cloned();
    let field_name = content_disposition
      .as_ref()
      .and_then(|cd| cd.get_name())
      .unwrap_or("")
      .to_string();

    if field_name != "file" {
      continue;
    }

    let filename = content_disposition
      .and_then(|cd| cd.get_filename().map(|s| s.to_string()))
      .unwrap_or_else(|| "file".to_string());
    let filename = sanitize_filename(&filename);
    let filename = unique_target_filename(&filename);

    let dir = user_files_dir(local_user_view.local_user.id.0);
    fs::create_dir_all(&dir).await?;
    let target = dir.join(&filename);
    let mut f = fs::File::create(&target).await?;

    let mut field = item;
    let mut size: u64 = 0;
    while let Some(chunk) = field.try_next().await.map_err(|_| FastJobErrorType::InvalidBodyField)? {
      size += chunk.len() as u64;
      if size > MAX_FILE_SIZE_BYTES {
        // Remove a partially written file
        let _ = fs::remove_file(&target).await;
        return Err(FastJobErrorType::InvalidBodyField.into());
      }
      f.write_all(&chunk).await?;
    }

    let protocol_and_hostname = context.settings().get_protocol_and_hostname();
    let url = file_url(local_user_view.local_user.id, &filename, &protocol_and_hostname)?;
    saved = Some(FileUploadResponse {
      filename,
      size,
      url,
    });
    break;
  }

  if let Some(resp) = saved {
    Ok(Json(resp))
  } else {
    Err(FastJobErrorType::InvalidBodyField.into())
  }
}
