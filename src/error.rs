use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PreviewError {
    #[error("Unsupported output format: {0}")]
    UnsupportedOutputFormat(String),
    
    #[error("Invalid input file: {0}")]
    InvalidInput(String),
    
    #[error("File processing failed: {0}")]
    ProcessingFailed(String),
    
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    
    #[error("Invalid file type or corrupted file")]
    InvalidFileType,
    
    #[error("Job not found: {0}")]
    JobNotFound(String),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
    
    #[error("External tool error: {0} - {1}")]
    ExternalToolError(String, String),
}

impl IntoResponse for PreviewError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            PreviewError::UnsupportedOutputFormat(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            PreviewError::InvalidInput(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            PreviewError::InvalidFileType => (StatusCode::BAD_REQUEST, self.to_string()),
            PreviewError::JobNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            PreviewError::DownloadFailed(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            PreviewError::ProcessingFailed(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            PreviewError::ExternalToolError(_, _) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(json!({
            "success": false,
            "error": message,
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, PreviewError>;