use crate::error::{PreviewError, Result};
use crate::preview::PreviewGenerator;
use crate::types::{
    JobState, JobStatus, PreviewRequest, PreviewResponse, ProcessingJob, JobStorage,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::{fs, sync::RwLock};
use tower_http::services::ServeFile;
use tracing::{error, info};
use uuid::Uuid;

// Global job storage (in production, use Redis or database)
type AppState = Arc<RwLock<JobStorage>>;

pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "service": "filepreview-rust",
        "version": "0.1.0"
    }))
}

pub async fn generate_preview(Json(request): Json<PreviewRequest>) -> Result<impl IntoResponse> {
    info!("Received preview request: {:?}", request.input);
    
    let generator = PreviewGenerator::new()?;
    
    // Handle input - could be URL or base64
    let input_path = if request.input.starts_with("http://") || request.input.starts_with("https://") {
        generator.download_file(&request.input).await?
    } else if request.input.starts_with("data:") || is_base64(&request.input) {
        // Extract filename from request or generate one
        let filename = format!("input.{}", guess_extension_from_base64(&request.input));
        generator.decode_base64_file(&request.input, &filename).await?
    } else {
        // Assume it's a file path (for testing)
        PathBuf::from(&request.input)
    };

    // Validate input file exists
    if !input_path.exists() {
        return Err(PreviewError::InvalidInput("File not found".to_string()));
    }

    // Generate preview
    let output_path = generator
        .generate_preview(&input_path, &request.output_format, &request.options)
        .await?;

    // Read the generated file and return as base64 (for simplicity)
    let output_bytes = fs::read(&output_path).await?;
    let base64_output = base64::encode(&output_bytes);
    
    let preview_url = format!(
        "data:image/{};base64,{}",
        request.output_format.extension(),
        base64_output
    );

    Ok(Json(PreviewResponse {
        success: true,
        message: "Preview generated successfully".to_string(),
        preview_url: Some(preview_url),
        job_id: None,
    }))
}

pub async fn generate_preview_async(
    State(jobs): State<AppState>,
    Json(request): Json<PreviewRequest>,
) -> Result<impl IntoResponse> {
    let job_id = Uuid::new_v4().to_string();
    
    info!("Received async preview request with job ID: {}", job_id);
    
    // Create job entry
    {
        let mut jobs_lock = jobs.write().await;
        jobs_lock.insert(
            job_id.clone(),
            ProcessingJob {
                id: job_id.clone(),
                status: JobState::Pending,
                message: "Job created".to_string(),
                result_path: None,
                progress: 0,
            },
        );
    }

    // Spawn background task to process the file
    let jobs_clone = jobs.clone();
    let job_id_clone = job_id.clone();
    tokio::spawn(async move {
        if let Err(e) = process_file_async(jobs_clone, job_id_clone, request).await {
            error!("Async processing failed: {}", e);
            
            // Update job status to failed
            let mut jobs_lock = jobs_clone.write().await;
            if let Some(job) = jobs_lock.get_mut(&job_id_clone) {
                job.status = JobState::Failed;
                job.message = e.to_string();
            }
        }
    });

    Ok(Json(PreviewResponse {
        success: true,
        message: "Job created successfully".to_string(),
        preview_url: None,
        job_id: Some(job_id),
    }))
}

pub async fn get_job_status(
    State(jobs): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<impl IntoResponse> {
    let jobs_lock = jobs.read().await;
    
    let job = jobs_lock
        .get(&job_id)
        .ok_or_else(|| PreviewError::JobNotFound(job_id.clone()))?;

    let result_url = if job.status == JobState::Completed {
        job.result_path.as_ref().map(|path| {
            format!("/download/{}", path.split('/').last().unwrap_or("result"))
        })
    } else {
        None
    };

    Ok(Json(JobStatus {
        job_id: job.id.clone(),
        status: job.status.clone(),
        message: job.message.clone(),
        result_url,
        progress: job.progress,
    }))
}

pub async fn download_file(Path(filename): Path<String>) -> Result<impl IntoResponse> {
    // In production, you'd want to validate the filename and path
    let file_path = format!("/tmp/{}", filename);
    
    if !PathBuf::from(&file_path).exists() {
        return Err(PreviewError::InvalidInput("File not found".to_string()));
    }
    
    Ok(ServeFile::new(file_path))
}

async fn process_file_async(
    jobs: AppState,
    job_id: String,
    request: PreviewRequest,
) -> Result<()> {
    // Update status to processing
    {
        let mut jobs_lock = jobs.write().await;
        if let Some(job) = jobs_lock.get_mut(&job_id) {
            job.status = JobState::Processing;
            job.message = "Processing file".to_string();
            job.progress = 10;
        }
    }

    let generator = PreviewGenerator::new()?;
    
    // Handle input
    let input_path = if request.input.starts_with("http://") || request.input.starts_with("https://") {
        // Update progress
        {
            let mut jobs_lock = jobs.write().await;
            if let Some(job) = jobs_lock.get_mut(&job_id) {
                job.message = "Downloading file".to_string();
                job.progress = 20;
            }
        }
        generator.download_file(&request.input).await?
    } else if request.input.starts_with("data:") || is_base64(&request.input) {
        let filename = format!("input.{}", guess_extension_from_base64(&request.input));
        generator.decode_base64_file(&request.input, &filename).await?
    } else {
        PathBuf::from(&request.input)
    };

    // Update progress
    {
        let mut jobs_lock = jobs.write().await;
        if let Some(job) = jobs_lock.get_mut(&job_id) {
            job.message = "Generating preview".to_string();
            job.progress = 50;
        }
    }

    // Generate preview
    let output_path = generator
        .generate_preview(&input_path, &request.output_format, &request.options)
        .await?;

    // Copy result to permanent location (in production, use cloud storage)
    let result_filename = format!("result_{}_{}.{}", 
        job_id, 
        chrono::Utc::now().timestamp(),
        request.output_format.extension()
    );
    let result_path = format!("/tmp/{}", result_filename);
    fs::copy(&output_path, &result_path).await?;

    // Update job to completed
    {
        let mut jobs_lock = jobs.write().await;
        if let Some(job) = jobs_lock.get_mut(&job_id) {
            job.status = JobState::Completed;
            job.message = "Preview generated successfully".to_string();
            job.result_path = Some(result_path);
            job.progress = 100;
        }
    }

    info!("Async job {} completed successfully", job_id);
    Ok(())
}

fn is_base64(s: &str) -> bool {
    // Simple heuristic: check if string looks like base64
    s.len() % 4 == 0 && s.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '=')
}

fn guess_extension_from_base64(data: &str) -> &'static str {
    if data.starts_with("data:") {
        if data.contains("image/png") {
            "png"
        } else if data.contains("image/jpeg") || data.contains("image/jpg") {
            "jpg"
        } else if data.contains("image/gif") {
            "gif"
        } else if data.contains("application/pdf") {
            "pdf"
        } else if data.contains("video/") {
            "mp4"
        } else {
            "bin"
        }
    } else {
        "bin"
    }
}