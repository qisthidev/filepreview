<?php

namespace App\Http\Controllers;

use App\Services\FilePreviewService;
use Illuminate\Http\Request;
use Illuminate\Http\JsonResponse;
use Illuminate\Support\Facades\Validator;
use Illuminate\Support\Facades\Storage;
use Illuminate\Support\Facades\Storage;

class FilePreviewController extends Controller
{
    private FilePreviewService $previewService;

    public function __construct(FilePreviewService $previewService)
    {
        $this->previewService = $previewService;
    }

    /**
     * Generate preview from uploaded file
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function uploadAndPreview(Request $request): JsonResponse
    {
        $validator = Validator::make($request->all(), [
            'file' => 'required|file|max:10240', // 10MB max
            'output_format' => 'in:gif,jpg,png',
            'width' => 'integer|min:1|max:2000',
            'height' => 'integer|min:1|max:2000',
            'quality' => 'integer|min:1|max:100',
            'preview_time' => 'string|regex:/^\d{2}:\d{2}:\d{2}\.\d{3}$/', // HH:MM:SS.mmm
        ]);

        if ($validator->fails()) {
            return response()->json([
                'success' => false,
                'errors' => $validator->errors(),
            ], 400);
        }

        try {
            $file = $request->file('file');
            $outputFormat = $request->get('output_format', 'jpg');
            
            $options = array_filter([
                'width' => $request->get('width'),
                'height' => $request->get('height'),
                'quality' => $request->get('quality'),
                'preview_time' => $request->get('preview_time'),
            ]);

            $result = $this->previewService->generatePreviewFromUploadedFile(
                $file, 
                $outputFormat, 
                $options
            );

            return response()->json($result);

        } catch (\Exception $e) {
            return response()->json([
                'success' => false,
                'error' => $e->getMessage(),
            ], 500);
        }
    }

    /**
     * Generate preview from URL
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function previewFromUrl(Request $request): JsonResponse
    {
        $validator = Validator::make($request->all(), [
            'url' => 'required|url',
            'output_format' => 'in:gif,jpg,png',
            'width' => 'integer|min:1|max:2000',
            'height' => 'integer|min:1|max:2000',
            'quality' => 'integer|min:1|max:100',
            'preview_time' => 'string|regex:/^\d{2}:\d{2}:\d{2}\.\d{3}$/',
        ]);

        if ($validator->fails()) {
            return response()->json([
                'success' => false,
                'errors' => $validator->errors(),
            ], 400);
        }

        try {
            $url = $request->get('url');
            $outputFormat = $request->get('output_format', 'jpg');
            
            $options = array_filter([
                'width' => $request->get('width'),
                'height' => $request->get('height'),
                'quality' => $request->get('quality'),
                'preview_time' => $request->get('preview_time'),
            ]);

            $result = $this->previewService->generatePreviewFromUrl(
                $url, 
                $outputFormat, 
                $options
            );

            return response()->json($result);

        } catch (\Exception $e) {
            return response()->json([
                'success' => false,
                'error' => $e->getMessage(),
            ], 500);
        }
    }

    /**
     * Generate preview asynchronously
     *
     * @param Request $request
     * @return JsonResponse
     */
    public function previewAsync(Request $request): JsonResponse
    {
        $validator = Validator::make($request->all(), [
            'input' => 'required|string',
            'input_type' => 'required|in:url,file,base64',
            'output_format' => 'in:gif,jpg,png',
            'width' => 'integer|min:1|max:2000',
            'height' => 'integer|min:1|max:2000',
            'quality' => 'integer|min:1|max:100',
            'preview_time' => 'string|regex:/^\d{2}:\d{2}:\d{2}\.\d{3}$/',
        ]);

        if ($validator->fails()) {
            return response()->json([
                'success' => false,
                'errors' => $validator->errors(),
            ], 400);
        }

        try {
            $input = $request->get('input');
            $inputType = $request->get('input_type');
            $outputFormat = $request->get('output_format', 'jpg');
            
            $options = array_filter([
                'width' => $request->get('width'),
                'height' => $request->get('height'),
                'quality' => $request->get('quality'),
                'preview_time' => $request->get('preview_time'),
            ]);

            // Handle different input types
            if ($inputType === 'file' && $request->hasFile('file')) {
                // Store file temporarily and get URL
                $file = $request->file('file');
                $path = $file->store('temp-uploads', 'public');
                $input = config('app.url') . Storage::url($path);
            }

            $result = $this->previewService->generatePreviewAsync(
                $input, 
                $outputFormat, 
                $options
            );

            return response()->json($result);

        } catch (\Exception $e) {
            return response()->json([
                'success' => false,
                'error' => $e->getMessage(),
            ], 500);
        }
    }

    /**
     * Check job status
     *
     * @param string $jobId
     * @return JsonResponse
     */
    public function getJobStatus(string $jobId): JsonResponse
    {
        try {
            $status = $this->previewService->getJobStatus($jobId);
            return response()->json($status);

        } catch (\Exception $e) {
            return response()->json([
                'success' => false,
                'error' => $e->getMessage(),
            ], 500);
        }
    }

    /**
     * Download completed job result
     *
     * @param string $jobId
     * @return \Illuminate\Http\Response|JsonResponse
     */
    public function downloadJobResult(string $jobId)
    {
        try {
            $content = $this->previewService->downloadResult($jobId);
            
            if ($content === null) {
                return response()->json([
                    'success' => false,
                    'error' => 'Job not completed or result not available',
                ], 404);
            }

            // Get job status to determine file type
            $status = $this->previewService->getJobStatus($jobId);
            $filename = "preview_{$jobId}.jpg"; // Default filename
            
            return response($content)
                ->header('Content-Type', 'image/jpeg')
                ->header('Content-Disposition', "attachment; filename=\"{$filename}\"");

        } catch (\Exception $e) {
            return response()->json([
                'success' => false,
                'error' => $e->getMessage(),
            ], 500);
        }
    }

    /**
     * Health check for the preview service
     *
     * @return JsonResponse
     */
    public function healthCheck(): JsonResponse
    {
        $healthy = $this->previewService->isHealthy();
        
        return response()->json([
            'service' => 'file-preview',
            'status' => $healthy ? 'healthy' : 'unhealthy',
            'timestamp' => now()->toISOString(),
        ], $healthy ? 200 : 503);
    }

    /**
     * Get service capabilities and supported formats
     *
     * @return JsonResponse
     */
    public function getCapabilities(): JsonResponse
    {
        return response()->json([
            'supported_input_formats' => [
                'images' => ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'tiff', 'webp', 'pdf'],
                'videos' => ['mp4', 'avi', 'mov', 'wmv', 'flv', 'webm', 'mkv'],
                'documents' => ['doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'odt', 'ods', 'odp', 'pdf'],
            ],
            'supported_output_formats' => ['gif', 'jpg', 'png'],
            'max_file_size' => '50MB',
            'input_methods' => ['file_upload', 'url', 'base64'],
            'processing_modes' => ['synchronous', 'asynchronous'],
            'options' => [
                'width' => 'integer (1-2000)',
                'height' => 'integer (1-2000)', 
                'quality' => 'integer (1-100)',
                'preview_time' => 'string (HH:MM:SS.mmm) - for videos only',
            ],
        ]);
    }
}