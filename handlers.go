package main

import (
	"fmt"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
)

type PreviewRequest struct {
	InputPath  string            `json:"input_path" binding:"required"`
	OutputPath string            `json:"output_path" binding:"required"`
	Options    *PreviewOptions   `json:"options,omitempty"`
}

type URLPreviewRequest struct {
	URL        string            `json:"url" binding:"required"`
	OutputPath string            `json:"output_path" binding:"required"`
	Options    *PreviewOptions   `json:"options,omitempty"`
}

type PreviewOptions struct {
	Width       int    `json:"width,omitempty"`
	Height      int    `json:"height,omitempty"`
	Quality     int    `json:"quality,omitempty"`
	PreviewTime string `json:"preview_time,omitempty"`
}

type PreviewResponse struct {
	Success    bool   `json:"success"`
	Message    string `json:"message,omitempty"`
	OutputPath string `json:"output_path,omitempty"`
	Error      string `json:"error,omitempty"`
}

// GeneratePreview handles file preview generation from local file
func GeneratePreview(c *gin.Context) {
	var req PreviewRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, PreviewResponse{
			Success: false,
			Error:   fmt.Sprintf("Invalid request: %v", err),
		})
		return
	}

	// Validate output format
	if !isValidOutputFormat(req.OutputPath) {
		c.JSON(http.StatusBadRequest, PreviewResponse{
			Success: false,
			Error:   "Unsupported output format. Only gif, jpg, and png are supported",
		})
		return
	}

	// Generate preview
	err := generateFilePreview(req.InputPath, req.OutputPath, req.Options)
	if err != nil {
		c.JSON(http.StatusInternalServerError, PreviewResponse{
			Success: false,
			Error:   fmt.Sprintf("Failed to generate preview: %v", err),
		})
		return
	}

	c.JSON(http.StatusOK, PreviewResponse{
		Success:    true,
		Message:    "Preview generated successfully",
		OutputPath: req.OutputPath,
	})
}

// GeneratePreviewFromURL handles file preview generation from URL
func GeneratePreviewFromURL(c *gin.Context) {
	var req URLPreviewRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, PreviewResponse{
			Success: false,
			Error:   fmt.Sprintf("Invalid request: %v", err),
		})
		return
	}

	// Validate output format
	if !isValidOutputFormat(req.OutputPath) {
		c.JSON(http.StatusBadRequest, PreviewResponse{
			Success: false,
			Error:   "Unsupported output format. Only gif, jpg, and png are supported",
		})
		return
	}

	// Generate preview from URL
	err := generateFilePreviewFromURL(req.URL, req.OutputPath, req.Options)
	if err != nil {
		c.JSON(http.StatusInternalServerError, PreviewResponse{
			Success: false,
			Error:   fmt.Sprintf("Failed to generate preview: %v", err),
		})
		return
	}

	c.JSON(http.StatusOK, PreviewResponse{
		Success:    true,
		Message:    "Preview generated successfully",
		OutputPath: req.OutputPath,
	})
}

// GetSupportedFormats returns list of supported input formats
func GetSupportedFormats(c *gin.Context) {
	formats := map[string][]string{
		"image": {
			"jpg", "jpeg", "png", "gif", "bmp", "tiff", "svg", "webp", "ico",
			"psd", "xcf", "raw", "cr2", "nef", "orf", "sr2", "dng",
		},
		"video": {
			"mp4", "avi", "mov", "wmv", "flv", "webm", "mkv", "m4v", "3gp",
			"ogv", "ts", "mts", "m2ts", "vob", "rm", "rmvb", "asf",
		},
		"document": {
			"pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods",
			"odp", "rtf", "txt", "csv", "html", "xml", "tex", "md",
		},
		"output": {
			"jpg", "png", "gif",
		},
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"formats": formats,
	})
}