# FilePreview API - Postman Collection

This Postman collection contains example endpoints for a conceptual REST API based on the FilePreview Node.js library. Since the original project is a library (not a web server), this collection demonstrates how the library's functionality could be exposed as HTTP endpoints.

## Collection Overview

The collection includes the following endpoint categories:

### 🎬 File Preview Generation
- **Generate Preview (Async)** - Generate file previews asynchronously
- **Generate Preview (Sync)** - Generate file previews synchronously  
- **Generate Preview from URL** - Generate previews from remote URLs

### 📁 File Upload & Preview
- **Upload File and Generate Preview** - Upload files and generate previews in one request

### 🔧 Utility Endpoints
- **Get Supported File Types** - List supported input and output formats
- **Health Check** - Check API status and dependencies
- **View Generated Preview** - Download or view generated preview files

### 📦 Batch Operations
- **Batch Generate Previews** - Process multiple files in a single request

## Supported File Types

Based on the FilePreview library capabilities:

### Input Types
- **Images**: JPG, JPEG, PNG, GIF, BMP, TIFF, PDF
- **Videos**: MP4, AVI, MOV, WMV, FLV, WebM
- **Documents**: PDF, DOC, DOCX, PPT, PPTX, XLS, XLSX

### Output Formats
- JPG, PNG, GIF

## Configuration

### Environment Variables
The collection uses the following variable:
- `baseUrl`: Base URL for the API (default: `http://localhost:3000`)

### Request Parameters

#### Common Options
```json
{
  "input": "/path/to/input/file.ext",
  "output": "/path/to/output/preview.jpg",
  "options": {
    "width": 300,
    "height": 200,
    "quality": 85,
    "previewTime": "00:00:05"
  }
}
```

#### Parameter Details
- **input**: Path to input file or URL (required)
- **output**: Path for output preview file (required)
- **width**: Preview width in pixels (optional)
- **height**: Preview height in pixels (optional)
- **quality**: Image quality 1-100 (optional)
- **previewTime**: Video timestamp for preview frame (optional, format: HH:MM:SS)

## Usage Instructions

1. **Import Collection**: Import `filepreview-api.postman_collection.json` into Postman

2. **Set Base URL**: Update the `baseUrl` variable to match your API server

3. **Test Endpoints**: Start with the Health Check endpoint to verify connectivity

4. **File Paths**: Update file paths in request bodies to match your environment

## Example Use Cases

### Generate Video Thumbnail
```bash
POST /api/preview/generate
{
  "input": "/videos/movie.mp4",
  "output": "/thumbnails/movie-thumb.jpg",
  "options": {
    "width": 640,
    "height": 360,
    "previewTime": "00:01:30"
  }
}
```

### Generate Document Preview
```bash
POST /api/preview/generate-sync
{
  "input": "/documents/report.pdf",
  "output": "/previews/report-preview.png",
  "options": {
    "width": 800,
    "height": 600,
    "quality": 90
  }
}
```

### Process Remote File
```bash
POST /api/preview/generate-from-url
{
  "input": "https://example.com/video.mp4",
  "output": "/previews/remote-video.jpg",
  "options": {
    "width": 400,
    "height": 300
  }
}
```

## Dependencies

The API would require these system dependencies:
- **FFmpeg** - For video processing
- **ImageMagick** - For image processing  
- **LibreOffice/unoconv** - For document processing

## Implementation Notes

This collection is conceptual and assumes:
- A web server wrapper around the FilePreview library
- Proper file handling and storage mechanisms
- Error handling and validation
- Security measures for file uploads

To implement this API, you would need to:
1. Create an Express.js or similar web server
2. Integrate the FilePreview library
3. Handle file uploads and storage
4. Implement the endpoint logic shown in this collection

## Testing

The collection includes basic tests for:
- Response status codes
- Response time validation
- JSON response format validation

## Error Responses

Expected error responses should include:
```json
{
  "success": false,
  "error": "Error message",
  "code": "ERROR_CODE"
}
```

## Security Considerations

When implementing this API:
- Validate file types and sizes
- Sanitize file paths
- Implement rate limiting
- Use secure file storage
- Add authentication if needed