<?php

/*
|--------------------------------------------------------------------------
| File Preview Routes
|--------------------------------------------------------------------------
|
| Add these routes to your Laravel routes/web.php or routes/api.php file
|
*/

use App\Http\Controllers\FilePreviewController;

Route::prefix('api/preview')->group(function () {
    
    // Health check
    Route::get('health', [FilePreviewController::class, 'healthCheck']);
    
    // Get service capabilities
    Route::get('capabilities', [FilePreviewController::class, 'getCapabilities']);
    
    // Synchronous preview generation
    Route::post('upload', [FilePreviewController::class, 'uploadAndPreview']);
    Route::post('url', [FilePreviewController::class, 'previewFromUrl']);
    
    // Asynchronous preview generation
    Route::post('async', [FilePreviewController::class, 'previewAsync']);
    Route::get('status/{jobId}', [FilePreviewController::class, 'getJobStatus']);
    Route::get('download/{jobId}', [FilePreviewController::class, 'downloadJobResult']);
    
});

/*
|--------------------------------------------------------------------------
| Usage Examples
|--------------------------------------------------------------------------
|
| Example API calls you can make to the Laravel endpoints:
|
| 1. Upload and generate preview:
|    POST /api/preview/upload
|    Form data: file (multipart), output_format=jpg, width=640, height=480
|
| 2. Generate preview from URL:
|    POST /api/preview/url
|    JSON: {"url": "https://example.com/document.pdf", "output_format": "png"}
|
| 3. Async processing:
|    POST /api/preview/async
|    JSON: {"input": "https://example.com/video.mp4", "input_type": "url", "preview_time": "00:01:30.000"}
|
| 4. Check job status:
|    GET /api/preview/status/{job-id}
|
| 5. Download result:
|    GET /api/preview/download/{job-id}
|
*/