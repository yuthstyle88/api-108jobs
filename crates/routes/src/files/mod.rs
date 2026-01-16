use app_108jobs_db_schema::newtypes::LocalUserId;
use serde::Serialize;
use url::Url;

pub mod upload;
pub mod download;
pub mod delete;

#[derive(Debug, Serialize)]
pub struct FileUploadResponse {
    pub filename: String,
    pub size: u64,
    pub url: Url,
}

#[derive(Debug)]
pub struct FilePath {
    pub user_id: i32,
    pub filename: String,
}


    pub fn file_url(local_user_id: LocalUserId, filename: &str, protocol_and_hostname: &str) -> Result<Url, url::ParseError> {
        Url::parse(&format!(
            "{protocol_and_hostname}/api/v4/files/{}/{}",
            local_user_id.0,
            filename,
        ))
    }
    pub fn delete_url(local_user_id: LocalUserId, filename: &str, protocol_and_hostname: &str) -> Result<Url, url::ParseError> {
        Url::parse(&format!(
            "{protocol_and_hostname}/api/v4/files/{}/{}",
            local_user_id.0,
            filename,
        ))
    }



#[derive(serde::Deserialize)]
pub struct DeleteFileRequest {
    pub filename: String,
}