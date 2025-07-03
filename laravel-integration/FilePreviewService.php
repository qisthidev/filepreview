<?php

namespace App\Services;

use GuzzleHttp\Client;
use GuzzleHttp\Exception\GuzzleException;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Storage;

class FilePreviewService
{
    private Client $httpClient;
    private string $serviceUrl;

    public function __construct()
    {
        $this->httpClient = new Client([
            'timeout' => 30,
            'connect_timeout' => 10,
        ]);
        
        $this->serviceUrl = config('services.filepreview.url', 'http://localhost:3000');
    }

    /**
     * Generate a file preview synchronously
     *
     * @param string $input File path, URL, or base64 data
     * @param string $outputFormat gif, jpg, or png
     * @param array $options Optional parameters (width, height, quality, previewTime)
     * @return array
     * @throws \Exception
     */
    public function generatePreview(string $input, string $outputFormat = 'jpg', array $options = []): array
    {
        try {
            $response = $this->httpClient->post($this->serviceUrl . '/preview', [
                'json' => [
                    'input' => $input,
                    'output_format' => $outputFormat,
                    'options' => $options,
                ],
                'headers' => [
                    'Content-Type' => 'application/json',
                    'Accept' => 'application/json',
                ],
            ]);

            $data = json_decode($response->getBody()->getContents(), true);

            if (!$data['success']) {
                throw new \Exception('Preview generation failed: ' . ($data['error'] ?? 'Unknown error'));
            }

            return $data;

        } catch (GuzzleException $e) {
            Log::error('File preview service error: ' . $e->getMessage());
            throw new \Exception('Failed to communicate with preview service: ' . $e->getMessage());
        }
    }

    /**
     * Generate a file preview asynchronously
     *
     * @param string $input File path, URL, or base64 data
     * @param string $outputFormat gif, jpg, or png
     * @param array $options Optional parameters
     * @return array Contains job_id for status checking
     * @throws \Exception
     */
    public function generatePreviewAsync(string $input, string $outputFormat = 'jpg', array $options = []): array
    {
        try {
            $response = $this->httpClient->post($this->serviceUrl . '/preview/async', [
                'json' => [
                    'input' => $input,
                    'output_format' => $outputFormat,
                    'options' => $options,
                ],
                'headers' => [
                    'Content-Type' => 'application/json',
                    'Accept' => 'application/json',
                ],
            ]);

            $data = json_decode($response->getBody()->getContents(), true);

            if (!$data['success']) {
                throw new \Exception('Async preview generation failed: ' . ($data['error'] ?? 'Unknown error'));
            }

            return $data;

        } catch (GuzzleException $e) {
            Log::error('File preview service error: ' . $e->getMessage());
            throw new \Exception('Failed to communicate with preview service: ' . $e->getMessage());
        }
    }

    /**
     * Check the status of an async job
     *
     * @param string $jobId
     * @return array
     * @throws \Exception
     */
    public function getJobStatus(string $jobId): array
    {
        try {
            $response = $this->httpClient->get($this->serviceUrl . '/preview/status/' . $jobId, [
                'headers' => [
                    'Accept' => 'application/json',
                ],
            ]);

            return json_decode($response->getBody()->getContents(), true);

        } catch (GuzzleException $e) {
            Log::error('File preview service error: ' . $e->getMessage());
            throw new \Exception('Failed to get job status: ' . $e->getMessage());
        }
    }

    /**
     * Generate preview from uploaded Laravel file
     *
     * @param \Illuminate\Http\UploadedFile $file
     * @param string $outputFormat
     * @param array $options
     * @return array
     * @throws \Exception
     */
    public function generatePreviewFromUploadedFile($file, string $outputFormat = 'jpg', array $options = []): array
    {
        // Store file temporarily and get URL
        $path = $file->store('temp-uploads', 'public');
        $url = Storage::url($path);
        $fullUrl = config('app.url') . $url;

        try {
            $result = $this->generatePreview($fullUrl, $outputFormat, $options);
            
            // Clean up temporary file
            Storage::disk('public')->delete($path);
            
            return $result;
            
        } catch (\Exception $e) {
            // Clean up temporary file even if preview fails
            Storage::disk('public')->delete($path);
            throw $e;
        }
    }

    /**
     * Generate preview from file content as base64
     *
     * @param string $fileContent
     * @param string $mimeType
     * @param string $outputFormat
     * @param array $options
     * @return array
     * @throws \Exception
     */
    public function generatePreviewFromContent(string $fileContent, string $mimeType, string $outputFormat = 'jpg', array $options = []): array
    {
        $base64 = base64_encode($fileContent);
        $dataUrl = "data:{$mimeType};base64,{$base64}";
        
        return $this->generatePreview($dataUrl, $outputFormat, $options);
    }

    /**
     * Generate preview from a URL
     *
     * @param string $url
     * @param string $outputFormat
     * @param array $options
     * @return array
     * @throws \Exception
     */
    public function generatePreviewFromUrl(string $url, string $outputFormat = 'jpg', array $options = []): array
    {
        return $this->generatePreview($url, $outputFormat, $options);
    }

    /**
     * Check if the service is healthy
     *
     * @return bool
     */
    public function isHealthy(): bool
    {
        try {
            $response = $this->httpClient->get($this->serviceUrl . '/health', [
                'timeout' => 5,
            ]);

            $data = json_decode($response->getBody()->getContents(), true);
            return $data['status'] === 'healthy';

        } catch (\Exception $e) {
            Log::warning('File preview service health check failed: ' . $e->getMessage());
            return false;
        }
    }

    /**
     * Download result file from async job
     *
     * @param string $jobId
     * @return string|null File contents or null if not ready
     * @throws \Exception
     */
    public function downloadResult(string $jobId): ?string
    {
        $status = $this->getJobStatus($jobId);
        
        if ($status['status'] !== 'completed' || !isset($status['result_url'])) {
            return null;
        }

        try {
            $response = $this->httpClient->get($this->serviceUrl . $status['result_url']);
            return $response->getBody()->getContents();

        } catch (GuzzleException $e) {
            Log::error('Failed to download result: ' . $e->getMessage());
            throw new \Exception('Failed to download result file: ' . $e->getMessage());
        }
    }
}