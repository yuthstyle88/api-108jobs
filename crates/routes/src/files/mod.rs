use serde::Serialize;

pub mod upload;
pub mod download;

#[derive(Debug, Serialize)]
pub struct FileUploadResponse {
    pub filename: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug)]
pub struct FilePath {
    pub user_id: i32,
    pub filename: String,
}