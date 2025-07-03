use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewRequest {
    /// Input file URL or base64 encoded file
    pub input: String,
    /// Output format (gif, jpg, png)
    pub output_format: OutputFormat,
    /// Optional preview options
    #[serde(default)]
    pub options: PreviewOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewOptions {
    /// Width of the preview image
    pub width: Option<u32>,
    /// Height of the preview image  
    pub height: Option<u32>,
    /// Quality of the output (1-100)
    pub quality: Option<u32>,
    /// Preview time for videos (format: HH:MM:SS.mmm)
    pub preview_time: Option<String>,
}

impl Default for PreviewOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            quality: None,
            preview_time: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Gif,
    Jpg,
    Png,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Gif => "gif",
            OutputFormat::Jpg => "jpg", 
            OutputFormat::Png => "png",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Image,
    Video,
    Document,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResponse {
    pub success: bool,
    pub message: String,
    /// URL to download the generated preview
    pub preview_url: Option<String>,
    /// For async requests, job ID to check status
    pub job_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    pub job_id: String,
    pub status: JobState,
    pub message: String,
    /// URL to download the result if completed
    pub result_url: Option<String>,
    /// Progress percentage (0-100)
    pub progress: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobState {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ProcessingJob {
    pub id: String,
    pub status: JobState,
    pub message: String,
    pub result_path: Option<String>,
    pub progress: u8,
}

// Global job storage (in production, use a proper database or Redis)
pub type JobStorage = HashMap<String, ProcessingJob>;