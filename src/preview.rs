use crate::error::{PreviewError, Result};
use crate::types::{FileType, OutputFormat, PreviewOptions};
use mime_guess::from_path;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, info};

pub struct PreviewGenerator {
    temp_dir: TempDir,
}

impl PreviewGenerator {
    pub fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()
            .map_err(|e| PreviewError::InternalError(format!("Failed to create temp directory: {}", e)))?;
        
        Ok(Self { temp_dir })
    }

    pub async fn generate_preview(
        &self,
        input_path: &Path,
        output_format: &OutputFormat,
        options: &PreviewOptions,
    ) -> Result<PathBuf> {
        let file_type = self.detect_file_type(input_path)?;
        let output_path = self.create_output_path(output_format)?;

        debug!("Processing file: {:?} as {:?}", input_path, file_type);

        match file_type {
            FileType::Image => self.process_image(input_path, &output_path, options).await?,
            FileType::Video => self.process_video(input_path, &output_path, options).await?,
            FileType::Document | FileType::Other => {
                self.process_document(input_path, &output_path, options).await?
            }
        }

        Ok(output_path)
    }

    pub async fn download_file(&self, url: &str) -> Result<PathBuf> {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(PreviewError::DownloadFailed(format!(
                "HTTP {}: {}",
                response.status(),
                url
            )));
        }

        // Extract filename from URL
        let url_path = url.split('/').last().unwrap_or("download");
        let file_path = self.temp_dir.path().join(url_path);
        
        let bytes = response.bytes().await?;
        fs::write(&file_path, bytes).await?;
        
        info!("Downloaded file: {:?}", file_path);
        Ok(file_path)
    }

    pub async fn decode_base64_file(&self, base64_data: &str, filename: &str) -> Result<PathBuf> {
        // Remove data URL prefix if present (e.g., "data:image/png;base64,")
        let base64_clean = if base64_data.starts_with("data:") {
            base64_data
                .split(',')
                .nth(1)
                .ok_or_else(|| PreviewError::InvalidInput("Invalid base64 data format".to_string()))?
        } else {
            base64_data
        };

        let bytes = base64::decode(base64_clean)
            .map_err(|e| PreviewError::InvalidInput(format!("Invalid base64: {}", e)))?;
        
        let file_path = self.temp_dir.path().join(filename);
        fs::write(&file_path, bytes).await?;
        
        info!("Decoded base64 file: {:?}", file_path);
        Ok(file_path)
    }

    fn detect_file_type(&self, path: &Path) -> Result<FileType> {
        let mime = from_path(path).first_or_octet_stream();
        let mime_str = mime.as_ref();
        
        let file_type = if mime_str.starts_with("image/") {
            FileType::Image
        } else if mime_str.starts_with("video/") {
            FileType::Video
        } else if mime_str == "application/pdf" {
            FileType::Image // PDF is handled like an image
        } else if mime_str.starts_with("application/") || mime_str.starts_with("text/") {
            FileType::Document
        } else {
            // Check by extension as fallback
            let ext = path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
                
            match ext.as_str() {
                "pdf" | "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" => FileType::Image,
                "mp4" | "avi" | "mov" | "wmv" | "flv" | "webm" | "mkv" => FileType::Video,
                "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp" => FileType::Document,
                _ => FileType::Other,
            }
        };
        
        debug!("Detected file type: {:?} for MIME: {}", file_type, mime_str);
        Ok(file_type)
    }

    fn create_output_path(&self, format: &OutputFormat) -> Result<PathBuf> {
        let filename = format!("preview_{}.{}", 
            hex::encode(&Sha256::digest(uuid::Uuid::new_v4().to_string())[..8]),
            format.extension()
        );
        Ok(self.temp_dir.path().join(filename))
    }

    async fn process_image(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: &PreviewOptions,
    ) -> Result<()> {
        let mut args = vec![format!("{}[0]", input_path.display())];
        
        if let (Some(width), Some(height)) = (options.width, options.height) {
            args.insert(0, "-resize".to_string());
            args.insert(1, format!("{}x{}", width, height));
        }
        
        if let Some(quality) = options.quality {
            args.insert(0, "-quality".to_string());
            args.insert(1, quality.to_string());
        }
        
        args.push(output_path.display().to_string());
        
        debug!("ImageMagick convert args: {:?}", args);
        
        let output = Command::new("convert")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PreviewError::ExternalToolError("convert".to_string(), e.to_string()))?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("ImageMagick convert failed: {}", stderr);
            return Err(PreviewError::ProcessingFailed(format!("ImageMagick failed: {}", stderr)));
        }
        
        info!("Successfully processed image: {:?}", output_path);
        Ok(())
    }

    async fn process_video(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: &PreviewOptions,
    ) -> Result<()> {
        let mut args = vec![
            "-y".to_string(),
            "-i".to_string(),
            input_path.display().to_string(),
            "-vf".to_string(),
        ];
        
        let video_filter = if let (Some(width), Some(height)) = (options.width, options.height) {
            format!("thumbnail,scale={}:{}", width, height)
        } else {
            "thumbnail".to_string()
        };
        
        args.push(video_filter);
        args.extend_from_slice(&["-frames:v".to_string(), "1".to_string()]);
        
        if let Some(preview_time) = &options.preview_time {
            args.extend_from_slice(&["-ss".to_string(), preview_time.clone()]);
        }
        
        args.push(output_path.display().to_string());
        
        debug!("FFmpeg args: {:?}", args);
        
        let output = Command::new("ffmpeg")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PreviewError::ExternalToolError("ffmpeg".to_string(), e.to_string()))?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("FFmpeg failed: {}", stderr);
            return Err(PreviewError::ProcessingFailed(format!("FFmpeg failed: {}", stderr)));
        }
        
        info!("Successfully processed video: {:?}", output_path);
        Ok(())
    }

    async fn process_document(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: &PreviewOptions,
    ) -> Result<()> {
        // First convert document to PDF using unoconv
        let pdf_path = self.temp_dir.path().join("temp_document.pdf");
        
        debug!("Converting document to PDF: {:?} -> {:?}", input_path, pdf_path);
        
        let unoconv_output = Command::new("unoconv")
            .args(&[
                "-e", "PageRange=1",
                "-o", &pdf_path.display().to_string(),
                &input_path.display().to_string(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PreviewError::ExternalToolError("unoconv".to_string(), e.to_string()))?;
            
        if !unoconv_output.status.success() {
            let stderr = String::from_utf8_lossy(&unoconv_output.stderr);
            error!("unoconv failed: {}", stderr);
            return Err(PreviewError::ProcessingFailed(format!("unoconv failed: {}", stderr)));
        }
        
        // Then convert PDF to image using ImageMagick
        let mut args = vec![format!("{}[0]", pdf_path.display())];
        
        if let (Some(width), Some(height)) = (options.width, options.height) {
            args.insert(0, "-resize".to_string());
            args.insert(1, format!("{}x{}", width, height));
        }
        
        if let Some(quality) = options.quality {
            args.insert(0, "-quality".to_string());
            args.insert(1, quality.to_string());
        }
        
        args.push(output_path.display().to_string());
        
        debug!("ImageMagick convert args for document: {:?}", args);
        
        let convert_output = Command::new("convert")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PreviewError::ExternalToolError("convert".to_string(), e.to_string()))?;
            
        if !convert_output.status.success() {
            let stderr = String::from_utf8_lossy(&convert_output.stderr);
            error!("ImageMagick convert failed: {}", stderr);
            return Err(PreviewError::ProcessingFailed(format!("ImageMagick failed: {}", stderr)));
        }
        
        info!("Successfully processed document: {:?}", output_path);
        Ok(())
    }

    pub fn temp_path(&self) -> &Path {
        self.temp_dir.path()
    }
}