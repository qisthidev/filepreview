# File Preview Service (Go)

A high-performance file preview generator service written in Go, designed to replace Node.js filepreview with better cloud compatibility and Laravel integration.

## Overview

This service generates image previews (thumbnails) for various file formats including:
- **Images**: JPG, PNG, GIF, BMP, TIFF, SVG, WebP, PSD, RAW formats, etc.
- **Videos**: MP4, AVI, MOV, WMV, FLV, WebM, MKV, etc.
- **Documents**: PDF, DOC, DOCX, XLS, XLSX, PPT, PPTX, ODT, etc.

**Output formats**: JPG, PNG, GIF

## Features

- ✅ **REST API** - Easy integration with any web framework
- ✅ **Laravel Compatible** - CORS-enabled for seamless Laravel integration
- ✅ **Cloud Ready** - Docker support and environment-based configuration
- ✅ **URL Support** - Generate previews from remote URLs
- ✅ **Customizable** - Width, height, quality, and timing options
- ✅ **High Performance** - Go's concurrency and speed
- ✅ **Health Checks** - Built-in health monitoring endpoint

## Quick Start

### Prerequisites

Install required system dependencies:

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y unoconv ffmpeg imagemagick curl

# CentOS/RHEL
sudo yum install -y unoconv ffmpeg ImageMagick curl

# macOS
brew install unoconv ffmpeg imagemagick
```

### Installation

1. **Clone and build:**
```bash
git clone <repository-url>
cd filepreview-go
go mod tidy
go build -o filepreview-service
```

2. **Run the service:**
```bash
./filepreview-service
```

The service will start on port 8080 by default.

### Docker Deployment

```bash
# Build the Docker image
docker build -t filepreview-go .

# Run the container
docker run -p 8080:8080 filepreview-go
```

## API Documentation

### Base URL
```
http://localhost:8080/api/v1
```

### Endpoints

#### 1. Generate Preview from Local File

**POST** `/api/v1/generate`

```json
{
  "input_path": "/path/to/input/file.pdf",
  "output_path": "/path/to/output/thumbnail.jpg",
  "options": {
    "width": 640,
    "height": 480,
    "quality": 90,
    "preview_time": "00:03:00.000"
  }
}
```

#### 2. Generate Preview from URL

**POST** `/api/v1/generate-from-url`

```json
{
  "url": "https://example.com/document.pdf",
  "output_path": "/path/to/output/thumbnail.jpg",
  "options": {
    "width": 640,
    "height": 480,
    "quality": 90
  }
}
```

#### 3. Get Supported Formats

**GET** `/api/v1/formats`

Returns list of supported input and output formats.

#### 4. Health Check

**GET** `/health`

Returns service health status.

### Request Options

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `width` | integer | Output width in pixels | Original |
| `height` | integer | Output height in pixels | Original |
| `quality` | integer | Output quality (1-100) | 75 |
| `preview_time` | string | Video timestamp (HH:MM:SS.mmm) | 00:00:01 |

### Response Format

```json
{
  "success": true,
  "message": "Preview generated successfully",
  "output_path": "/path/to/output/thumbnail.jpg"
}
```

**Error Response:**
```json
{
  "success": false,
  "error": "Error description"
}
```

## Laravel Integration

### 1. Create Laravel Service Class

```php
<?php

namespace App\Services;

use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;

class FilePreviewService
{
    private string $baseUrl;

    public function __construct()
    {
        $this->baseUrl = config('services.filepreview.url', 'http://localhost:8080/api/v1');
    }

    /**
     * Generate preview from local file
     */
    public function generatePreview(string $inputPath, string $outputPath, array $options = []): bool
    {
        try {
            $response = Http::timeout(60)->post("{$this->baseUrl}/generate", [
                'input_path' => $inputPath,
                'output_path' => $outputPath,
                'options' => $options
            ]);

            return $response->successful() && $response->json('success', false);
        } catch (\Exception $e) {
            Log::error('File preview generation failed', [
                'input' => $inputPath,
                'error' => $e->getMessage()
            ]);
            return false;
        }
    }

    /**
     * Generate preview from URL
     */
    public function generatePreviewFromUrl(string $url, string $outputPath, array $options = []): bool
    {
        try {
            $response = Http::timeout(60)->post("{$this->baseUrl}/generate-from-url", [
                'url' => $url,
                'output_path' => $outputPath,
                'options' => $options
            ]);

            return $response->successful() && $response->json('success', false);
        } catch (\Exception $e) {
            Log::error('File preview generation from URL failed', [
                'url' => $url,
                'error' => $e->getMessage()
            ]);
            return false;
        }
    }

    /**
     * Get supported formats
     */
    public function getSupportedFormats(): array
    {
        try {
            $response = Http::get("{$this->baseUrl}/formats");
            return $response->successful() ? $response->json('formats', []) : [];
        } catch (\Exception $e) {
            Log::error('Failed to get supported formats', ['error' => $e->getMessage()]);
            return [];
        }
    }
}
```

### 2. Laravel Configuration

Add to `config/services.php`:

```php
'filepreview' => [
    'url' => env('FILEPREVIEW_SERVICE_URL', 'http://localhost:8080/api/v1'),
],
```

Add to `.env`:

```env
FILEPREVIEW_SERVICE_URL=http://your-go-service:8080/api/v1
```

### 3. Laravel Controller Example

```php
<?php

namespace App\Http\Controllers;

use App\Services\FilePreviewService;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Storage;

class DocumentController extends Controller
{
    private FilePreviewService $previewService;

    public function __construct(FilePreviewService $previewService)
    {
        $this->previewService = $previewService;
    }

    public function uploadAndPreview(Request $request)
    {
        $request->validate([
            'file' => 'required|file|max:10240', // 10MB max
        ]);

        $file = $request->file('file');
        $filename = time() . '_' . $file->getClientOriginalName();
        $inputPath = $file->storeAs('documents', $filename, 'public');
        $outputPath = Storage::path('public/thumbnails/' . pathinfo($filename, PATHINFO_FILENAME) . '.jpg');

        // Ensure thumbnail directory exists
        Storage::makeDirectory('public/thumbnails');

        // Generate preview
        $success = $this->previewService->generatePreview(
            Storage::path('public/' . $inputPath),
            $outputPath,
            [
                'width' => 300,
                'height' => 200,
                'quality' => 85
            ]
        );

        if ($success) {
            return response()->json([
                'success' => true,
                'file_url' => Storage::url($inputPath),
                'thumbnail_url' => Storage::url('thumbnails/' . pathinfo($filename, PATHINFO_FILENAME) . '.jpg')
            ]);
        }

        return response()->json(['success' => false, 'error' => 'Preview generation failed'], 500);
    }
}
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | Service port | 8080 |
| `GIN_MODE` | Gin framework mode (debug/release) | release |

## Docker Configuration

### Dockerfile

```dockerfile
FROM golang:1.21-alpine AS builder

WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download

COPY . .
RUN go build -o filepreview-service .

FROM alpine:latest

# Install system dependencies
RUN apk add --no-cache \
    unoconv \
    ffmpeg \
    imagemagick \
    curl \
    libreoffice

WORKDIR /app
COPY --from=builder /app/filepreview-service .

EXPOSE 8080

CMD ["./filepreview-service"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  filepreview:
    build: .
    ports:
      - "8080:8080"
    environment:
      - GIN_MODE=release
      - PORT=8080
    volumes:
      - ./uploads:/app/uploads
      - ./thumbnails:/app/thumbnails
    restart: unless-stopped

  # Optional: Add Redis for caching
  redis:
    image: redis:alpine
    ports:
      - "6379:6379"
    restart: unless-stopped
```

## Cloud Deployment

### AWS ECS

1. **Build and push to ECR:**
```bash
aws ecr get-login-password --region us-west-2 | docker login --username AWS --password-stdin <account>.dkr.ecr.us-west-2.amazonaws.com
docker build -t filepreview-go .
docker tag filepreview-go:latest <account>.dkr.ecr.us-west-2.amazonaws.com/filepreview-go:latest
docker push <account>.dkr.ecr.us-west-2.amazonaws.com/filepreview-go:latest
```

2. **ECS Task Definition:**
```json
{
  "family": "filepreview-go",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "512",
  "memory": "1024",
  "containerDefinitions": [
    {
      "name": "filepreview",
      "image": "<account>.dkr.ecr.us-west-2.amazonaws.com/filepreview-go:latest",
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "GIN_MODE",
          "value": "release"
        }
      ]
    }
  ]
}
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: filepreview-go
spec:
  replicas: 3
  selector:
    matchLabels:
      app: filepreview-go
  template:
    metadata:
      labels:
        app: filepreview-go
    spec:
      containers:
      - name: filepreview
        image: your-registry/filepreview-go:latest
        ports:
        - containerPort: 8080
        env:
        - name: GIN_MODE
          value: "release"
        resources:
          limits:
            memory: "1Gi"
            cpu: "500m"
          requests:
            memory: "512Mi"
            cpu: "250m"
---
apiVersion: v1
kind: Service
metadata:
  name: filepreview-service
spec:
  selector:
    app: filepreview-go
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: LoadBalancer
```

## Performance Considerations

- **Concurrency**: Go handles concurrent requests efficiently
- **Memory**: Large files are processed in chunks to minimize memory usage
- **Temporary Files**: Automatic cleanup of temporary files
- **Timeouts**: Configure appropriate timeouts for large file processing

## Troubleshooting

### Common Issues

1. **"unoconv failed" error**: Ensure LibreOffice is installed and unoconv is in PATH
2. **"ffmpeg failed" error**: Verify ffmpeg installation and codec support
3. **"convert failed" error**: Check ImageMagick installation and policies
4. **Permission errors**: Ensure write permissions for output directories

### Debugging

Enable debug mode:
```bash
GIN_MODE=debug ./filepreview-service
```

### Health Check

```bash
curl http://localhost:8080/health
```

## Migration from Node.js filepreview

### API Differences

| Node.js | Go Service | Notes |
|---------|------------|-------|
| `filepreview.generate()` | `POST /api/v1/generate` | Now HTTP-based |
| `filepreview.generateSync()` | Same endpoint | All operations are async |
| Callback-based | HTTP response | Standard REST API |

### Code Migration Example

**Before (Node.js):**
```javascript
var filepreview = require('filepreview');

filepreview.generate('/home/myfile.docx', '/home/myfile_preview.gif', function(error) {
  if (error) {
    return console.log(error);
  }
  console.log('File preview is /home/myfile_preview.gif');
});
```

**After (Laravel with Go service):**
```php
$previewService = new FilePreviewService();
$success = $previewService->generatePreview(
    '/home/myfile.docx',
    '/home/myfile_preview.gif'
);

if ($success) {
    echo 'File preview is /home/myfile_preview.gif';
} else {
    echo 'Preview generation failed';
}
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the BSD-4-Clause License - see the LICENSE file for details.

## Support

For issues and questions:
- Create an issue on GitHub
- Check the troubleshooting section
- Review system dependencies

---

**Note**: This service requires system dependencies (unoconv, ffmpeg, imagemagick) to be installed on the host system or container.
