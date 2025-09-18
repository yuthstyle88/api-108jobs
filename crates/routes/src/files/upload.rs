use actix_multipart::Multipart;
use actix_web::web::{Data, Json};
use futures_util::TryStreamExt as StreamExt;
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{FastJobErrorType, FastJobResult};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::{fs, io::AsyncWriteExt};

#[derive(Debug, Serialize)]
pub struct FileUploadResponse {
  pub filename: String,
  pub size: u64,
  pub url: String,
}

const MAX_FILE_SIZE_BYTES: u64 = 25 * 1024 * 1024; // 20 MB

fn sanitize_filename(name: &str) -> String {
  let name = name.trim();
  // Strip any path components and keep only a-zA-Z0-9 . _ -
  let base = Path::new(name).file_name().unwrap_or_default().to_string_lossy();
  base
    .chars()
    .map(|c| match c {
      'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | '-' => c,
      _ => '-'
    })
    .collect()
}

fn unique_target_filename(orig: &str) -> String {
  let ts = chrono::Utc::now().timestamp_millis();
  if let Some((stem, ext)) = orig.rsplit_once('.') {
    format!("{}_{}.{}", stem, ts, ext)
  } else {
    format!("{}_{}", orig, ts)
  }
}

fn user_files_dir(user_id: i32) -> PathBuf {
  PathBuf::from("uploads").join("files").join(user_id.to_string())
}

pub async fn upload_file(
  mut payload: Multipart,
  local_user_view: LocalUserView,
  _context: Data<FastJobContext>,
) -> FastJobResult<Json<FileUploadResponse>> {
  // Only first file field is considered
  let mut saved: Option<FileUploadResponse> = None;

  while let Some(item) = payload.try_next().await.map_err(|_| FastJobErrorType::InvalidBodyField)? {
    let content_disposition = item.content_disposition().cloned();
    let field_name = content_disposition
      .as_ref()
      .and_then(|cd| cd.get_name())
      .unwrap_or("")
      .to_string();

    if field_name != "file" {
      // skip unknown fields
      continue;
    }

    let filename = content_disposition
      .and_then(|cd| cd.get_filename().map(|s| s.to_string()))
      .unwrap_or_else(|| "file".to_string());
    let filename = sanitize_filename(&filename);
    let filename = unique_target_filename(&filename);

    let dir = user_files_dir(local_user_view.person.id.0);
    fs::create_dir_all(&dir).await?;
    let target = dir.join(&filename);
    let mut f = fs::File::create(&target).await?;

    let mut field = item;
    let mut size: u64 = 0;
    while let Some(chunk) = field.try_next().await.map_err(|_| FastJobErrorType::InvalidBodyField)? {
      size += chunk.len() as u64;
      if size > MAX_FILE_SIZE_BYTES {
        // Remove partially written file
        let _ = fs::remove_file(&target).await;
        return Err(FastJobErrorType::InvalidBodyField.into());
      }
      f.write_all(&chunk).await?;
    }

    let url = format!("/api/v4/files/{}/{}", local_user_view.person.id.0, filename);
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
