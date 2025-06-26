package mcp

import (
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strings"

	"github.com/rs/zerolog"
)

// SecureFilesystem provides secure, sandboxed filesystem operations using Go 1.24's os.Root
type SecureFilesystem struct {
	root   *os.Root
	logger zerolog.Logger
	config *SecurityConfig
}

// SecurityConfig defines security policies for filesystem operations
type SecurityConfig struct {
	AllowedPaths     []string // Paths that are explicitly allowed
	ReadOnlyPaths    []string // Paths that are read-only
	DeniedPaths      []string // Paths that are explicitly denied
	MaxFileSize      int64    // Maximum file size in bytes
	AllowedExtensions []string // Allowed file extensions
	RequireConfirmation []string // Operations requiring user confirmation
}

// FileOperation represents different types of file operations
type FileOperation string

const (
	FileRead   FileOperation = "read"
	FileWrite  FileOperation = "write"
	FileDelete FileOperation = "delete"
	FileCreate FileOperation = "create"
	FileMkdir  FileOperation = "mkdir"
	FileStat   FileOperation = "stat"
)

// PermissionError represents a security permission error
type PermissionError struct {
	Operation FileOperation
	Path      string
	Reason    string
}

func (e *PermissionError) Error() string {
	return fmt.Sprintf("permission denied for %s on %s: %s", e.Operation, e.Path, e.Reason)
}

// NewSecureFilesystem creates a new secure filesystem instance
func NewSecureFilesystem(rootPath string, config *SecurityConfig, logger zerolog.Logger) (*SecureFilesystem, error) {
	// Use Go 1.24's os.Root for secure directory-limited access
	root, err := os.OpenRoot(rootPath)
	if err != nil {
		return nil, fmt.Errorf("failed to create secure filesystem root: %w", err)
	}

	return &SecureFilesystem{
		root:   root,
		logger: logger.With().Str("component", "secure_filesystem").Str("root", rootPath).Logger(),
		config: config,
	}, nil
}

// CheckPermission validates if an operation is allowed on a path
func (sf *SecureFilesystem) CheckPermission(operation FileOperation, path string) error {
	// Normalize path
	cleanPath := filepath.Clean(path)
	
	// Check if path is explicitly denied
	for _, denied := range sf.config.DeniedPaths {
		if sf.pathMatches(cleanPath, denied) {
			return &PermissionError{
				Operation: operation,
				Path:      path,
				Reason:    "path is explicitly denied",
			}
		}
	}

	// Check read-only restrictions for write operations
	if operation == FileWrite || operation == FileDelete || operation == FileCreate {
		for _, readonly := range sf.config.ReadOnlyPaths {
			if sf.pathMatches(cleanPath, readonly) {
				return &PermissionError{
					Operation: operation,
					Path:      path,
					Reason:    "path is read-only",
				}
			}
		}
	}

	// Check file extension restrictions
	if operation == FileCreate || operation == FileWrite {
		ext := strings.ToLower(filepath.Ext(cleanPath))
		if len(sf.config.AllowedExtensions) > 0 && !sf.contains(sf.config.AllowedExtensions, ext) {
			return &PermissionError{
				Operation: operation,
				Path:      path,
				Reason:    fmt.Sprintf("file extension %s not allowed", ext),
			}
		}
	}

	// Check if path is in allowed paths (if specified)
	if len(sf.config.AllowedPaths) > 0 {
		allowed := false
		for _, allowedPath := range sf.config.AllowedPaths {
			if sf.pathMatches(cleanPath, allowedPath) {
				allowed = true
				break
			}
		}
		if !allowed {
			return &PermissionError{
				Operation: operation,
				Path:      path,
				Reason:    "path not in allowed paths",
			}
		}
	}

	return nil
}

// SecureOpen opens a file with security checks
func (sf *SecureFilesystem) SecureOpen(name string) (*os.File, error) {
	if err := sf.CheckPermission(FileRead, name); err != nil {
		sf.logger.Warn().Err(err).Str("path", name).Msg("File access denied")
		return nil, err
	}

	sf.logger.Debug().Str("path", name).Msg("Opening file")
	return sf.root.Open(name)
}

// SecureCreate creates a file with security checks
func (sf *SecureFilesystem) SecureCreate(name string) (*os.File, error) {
	if err := sf.CheckPermission(FileCreate, name); err != nil {
		sf.logger.Warn().Err(err).Str("path", name).Msg("File creation denied")
		return nil, err
	}

	sf.logger.Info().Str("path", name).Msg("Creating file")
	return sf.root.Create(name)
}

// SecureRemove removes a file with security checks
func (sf *SecureFilesystem) SecureRemove(name string) error {
	if err := sf.CheckPermission(FileDelete, name); err != nil {
		sf.logger.Warn().Err(err).Str("path", name).Msg("File deletion denied")
		return err
	}

	sf.logger.Info().Str("path", name).Msg("Removing file")
	return sf.root.Remove(name)
}

// SecureMkdir creates a directory with security checks
func (sf *SecureFilesystem) SecureMkdir(name string) error {
	if err := sf.CheckPermission(FileMkdir, name); err != nil {
		sf.logger.Warn().Err(err).Str("path", name).Msg("Directory creation denied")
		return err
	}

	sf.logger.Info().Str("path", name).Msg("Creating directory")
	return sf.root.Mkdir(name, 0755)
}

// SecureStat gets file info with security checks
func (sf *SecureFilesystem) SecureStat(name string) (fs.FileInfo, error) {
	if err := sf.CheckPermission(FileStat, name); err != nil {
		sf.logger.Warn().Err(err).Str("path", name).Msg("File stat denied")
		return nil, err
	}

	sf.logger.Debug().Str("path", name).Msg("Getting file info")
	return sf.root.Stat(name)
}

// SecureReadDir reads directory contents with security checks
func (sf *SecureFilesystem) SecureReadDir(name string) ([]fs.DirEntry, error) {
	if err := sf.CheckPermission(FileRead, name); err != nil {
		sf.logger.Warn().Err(err).Str("path", name).Msg("Directory read denied")
		return nil, err
	}

	sf.logger.Debug().Str("path", name).Msg("Reading directory")
	file, err := sf.root.Open(name)
	if err != nil {
		return nil, err
	}
	defer file.Close()
	
	return file.ReadDir(-1)
}

// CheckFileSize validates file size against limits
func (sf *SecureFilesystem) CheckFileSize(name string) error {
	if sf.config.MaxFileSize <= 0 {
		return nil // No size limit
	}

	info, err := sf.root.Stat(name)
	if err != nil {
		return err
	}

	if info.Size() > sf.config.MaxFileSize {
		return &PermissionError{
			Operation: FileRead,
			Path:      name,
			Reason:    fmt.Sprintf("file size %d exceeds limit %d", info.Size(), sf.config.MaxFileSize),
		}
	}

	return nil
}

// RequiresConfirmation checks if an operation requires user confirmation
func (sf *SecureFilesystem) RequiresConfirmation(operation FileOperation) bool {
	for _, confirmedOp := range sf.config.RequireConfirmation {
		if confirmedOp == string(operation) {
			return true
		}
	}
	return false
}

// Close closes the secure filesystem root
func (sf *SecureFilesystem) Close() error {
	if sf.root != nil {
		return sf.root.Close()
	}
	return nil
}

// Helper methods

// pathMatches checks if a path matches a pattern (supports basic wildcards)
func (sf *SecureFilesystem) pathMatches(path, pattern string) bool {
	// Simple prefix matching for now
	// Could be enhanced with proper glob matching
	if strings.HasSuffix(pattern, "*") {
		prefix := strings.TrimSuffix(pattern, "*")
		return strings.HasPrefix(path, prefix)
	}
	
	// Exact match or subdirectory match
	return path == pattern || strings.HasPrefix(path, pattern+string(filepath.Separator))
}

// contains checks if a slice contains a string
func (sf *SecureFilesystem) contains(slice []string, item string) bool {
	for _, s := range slice {
		if s == item {
			return true
		}
	}
	return false
}

// GetSecurityStats returns security operation statistics
func (sf *SecureFilesystem) GetSecurityStats() map[string]interface{} {
	return map[string]interface{}{
		"allowed_paths":       len(sf.config.AllowedPaths),
		"readonly_paths":      len(sf.config.ReadOnlyPaths),
		"denied_paths":        len(sf.config.DeniedPaths),
		"allowed_extensions":  len(sf.config.AllowedExtensions),
		"max_file_size":       sf.config.MaxFileSize,
		"confirmation_required": len(sf.config.RequireConfirmation),
	}
}

// DefaultSecurityConfig returns a secure default configuration
func DefaultSecurityConfig() *SecurityConfig {
	return &SecurityConfig{
		AllowedPaths: []string{"."},
		ReadOnlyPaths: []string{
			".git",
			"node_modules",
			"vendor",
			".env",
		},
		DeniedPaths: []string{
			"/etc",
			"/usr",
			"/bin",
			"/sbin",
			"/var",
			"/tmp",
		},
		MaxFileSize: 10 * 1024 * 1024, // 10MB
		AllowedExtensions: []string{
			".go", ".js", ".ts", ".py", ".rs", ".java",
			".c", ".cpp", ".h", ".hpp",
			".md", ".txt", ".json", ".yaml", ".toml",
			".sql", ".sh", ".bat",
		},
		RequireConfirmation: []string{
			"delete", "write", "create",
		},
	}
}