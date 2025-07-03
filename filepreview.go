package main

import (
	"crypto/sha512"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

// FileType represents the type of file being processed
type FileType string

const (
	FileTypeImage FileType = "image"
	FileTypeVideo FileType = "video"
	FileTypeOther FileType = "other"
)

// isValidOutputFormat checks if the output format is supported
func isValidOutputFormat(outputPath string) bool {
	ext := strings.ToLower(filepath.Ext(outputPath))
	return ext == ".gif" || ext == ".jpg" || ext == ".jpeg" || ext == ".png"
}

// determineFileType determines the type of file based on extension
func determineFileType(inputPath string) FileType {
	ext := strings.ToLower(filepath.Ext(inputPath))
	ext = strings.TrimPrefix(ext, ".")

	imageExts := map[string]bool{
		"jpg": true, "jpeg": true, "png": true, "gif": true, "bmp": true,
		"tiff": true, "tif": true, "svg": true, "webp": true, "ico": true,
		"psd": true, "xcf": true, "raw": true, "cr2": true, "nef": true,
		"orf": true, "sr2": true, "dng": true, "pdf": true,
	}

	videoExts := map[string]bool{
		"mp4": true, "avi": true, "mov": true, "wmv": true, "flv": true,
		"webm": true, "mkv": true, "m4v": true, "3gp": true, "ogv": true,
		"ts": true, "mts": true, "m2ts": true, "vob": true, "rm": true,
		"rmvb": true, "asf": true,
	}

	if imageExts[ext] {
		return FileTypeImage
	}
	if videoExts[ext] {
		return FileTypeVideo
	}
	return FileTypeOther
}

// downloadFile downloads a file from URL to a temporary location
func downloadFile(url, destPath string) error {
	resp, err := http.Get(url)
	if err != nil {
		return fmt.Errorf("failed to download file: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("failed to download file: HTTP %d", resp.StatusCode)
	}

	out, err := os.Create(destPath)
	if err != nil {
		return fmt.Errorf("failed to create destination file: %v", err)
	}
	defer out.Close()

	_, err = io.Copy(out, resp.Body)
	if err != nil {
		return fmt.Errorf("failed to save file: %v", err)
	}

	return nil
}

// generateTempPath generates a temporary file path
func generateTempPath(suffix string) string {
	hash := sha512.New()
	hash.Write([]byte(fmt.Sprintf("%d", os.Getpid())))
	hashStr := fmt.Sprintf("%x", hash.Sum(nil))[:16]
	return filepath.Join(os.TempDir(), hashStr+suffix)
}

// generateFilePreviewFromURL downloads file from URL and generates preview
func generateFilePreviewFromURL(url, outputPath string, options *PreviewOptions) error {
	// Extract filename from URL
	urlParts := strings.Split(url, "/")
	filename := urlParts[len(urlParts)-1]
	if filename == "" {
		filename = "download"
	}

	// Generate temporary input path
	tempInput := generateTempPath("_" + filename)
	defer os.Remove(tempInput)

	// Download file
	if err := downloadFile(url, tempInput); err != nil {
		return err
	}

	// Generate preview from the downloaded file
	return generateFilePreview(tempInput, outputPath, options)
}

// generateFilePreview generates a preview for the given file
func generateFilePreview(inputPath, outputPath string, options *PreviewOptions) error {
	// Check if input file exists
	if _, err := os.Stat(inputPath); os.IsNotExist(err) {
		return fmt.Errorf("input file does not exist: %s", inputPath)
	}

	// Determine file type
	fileType := determineFileType(inputPath)

	// Set default options if nil
	if options == nil {
		options = &PreviewOptions{}
	}

	switch fileType {
	case FileTypeVideo:
		return generateVideoPreview(inputPath, outputPath, options)
	case FileTypeImage:
		return generateImagePreview(inputPath, outputPath, options)
	case FileTypeOther:
		return generateDocumentPreview(inputPath, outputPath, options)
	default:
		return fmt.Errorf("unsupported file type")
	}
}

// generateVideoPreview generates preview for video files using ffmpeg
func generateVideoPreview(inputPath, outputPath string, options *PreviewOptions) error {
	args := []string{"-y", "-i", inputPath, "-vf", "thumbnail", "-frames:v", "1"}

	// Add scaling if dimensions are specified
	if options.Width > 0 && options.Height > 0 {
		args[4] = fmt.Sprintf("thumbnail,scale=%d:%d", options.Width, options.Height)
	}

	// Add preview time if specified
	if options.PreviewTime != "" {
		args = append(args, "-ss", options.PreviewTime)
	}

	args = append(args, outputPath)

	cmd := exec.Command("ffmpeg", args...)
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("ffmpeg failed: %v", err)
	}

	return nil
}

// generateImagePreview generates preview for image files using ImageMagick
func generateImagePreview(inputPath, outputPath string, options *PreviewOptions) error {
	args := []string{}

	// Add quality if specified
	if options.Quality > 0 {
		args = append(args, "-quality", fmt.Sprintf("%d", options.Quality))
	}

	// Add resize if dimensions are specified
	if options.Width > 0 && options.Height > 0 {
		args = append(args, "-resize", fmt.Sprintf("%dx%d", options.Width, options.Height))
	}

	// Add input and output
	args = append(args, inputPath+"[0]", outputPath)

	cmd := exec.Command("convert", args...)
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("imagemagick convert failed: %v", err)
	}

	return nil
}

// generateDocumentPreview generates preview for document files using unoconv and ImageMagick
func generateDocumentPreview(inputPath, outputPath string, options *PreviewOptions) error {
	// Generate temporary PDF
	tempPDF := generateTempPath(".pdf")
	defer os.Remove(tempPDF)

	// Convert document to PDF using unoconv
	unoconvArgs := []string{"-e", "PageRange=1", "-o", tempPDF, inputPath}
	cmd := exec.Command("unoconv", unoconvArgs...)
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("unoconv failed: %v", err)
	}

	// Convert PDF to image using ImageMagick
	args := []string{}

	// Add quality if specified
	if options.Quality > 0 {
		args = append(args, "-quality", fmt.Sprintf("%d", options.Quality))
	}

	// Add resize if dimensions are specified
	if options.Width > 0 && options.Height > 0 {
		args = append(args, "-resize", fmt.Sprintf("%dx%d", options.Width, options.Height))
	}

	// Add input and output
	args = append(args, tempPDF+"[0]", outputPath)

	cmd = exec.Command("convert", args...)
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("imagemagick convert failed: %v", err)
	}

	return nil
}