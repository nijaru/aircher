package analysis

import (
	"bufio"
	"crypto/sha256"
	"encoding/json"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"time"

	"github.com/aircher/aircher/internal/storage"
	"github.com/rs/zerolog"
)

// ProjectAnalyzer analyzes project structure and documentation
type ProjectAnalyzer struct {
	projectRoot   string
	storageEngine *storage.Engine
	logger        zerolog.Logger
}

// DocumentationFile represents a discovered documentation file
type DocumentationFile struct {
	FilePath     string    `json:"file_path"`
	DocType      string    `json:"doc_type"`
	Title        string    `json:"title"`
	Purpose      string    `json:"purpose"`
	Sections     []string  `json:"sections"`
	CrossRefs    []string  `json:"cross_refs"`
	LastAnalyzed time.Time `json:"last_analyzed"`
	ContentHash  string    `json:"content_hash"`
}

// ProjectComponent represents a discovered project component
type ProjectComponent struct {
	Component   string                 `json:"component"`
	Type        string                 `json:"type"`
	Description string                 `json:"description"`
	Confidence  float64                `json:"confidence"`
	Evidence    []string               `json:"evidence"`
	Metadata    map[string]interface{} `json:"metadata"`
	LastUpdated time.Time              `json:"last_updated"`
}

// ProjectMetadata represents key-value project information
type ProjectMetadata struct {
	Key         string    `json:"key"`
	Value       string    `json:"value"`
	Confidence  float64   `json:"confidence"`
	Source      string    `json:"source"`
	Category    string    `json:"category"`
	LastUpdated time.Time `json:"last_updated"`
}

// AnalysisResult contains the complete project analysis
type AnalysisResult struct {
	Documentation []DocumentationFile `json:"documentation"`
	Components    []ProjectComponent  `json:"components"`
	Metadata      []ProjectMetadata   `json:"metadata"`
	AnalyzedAt    time.Time           `json:"analyzed_at"`
}

// NewProjectAnalyzer creates a new project analyzer
func NewProjectAnalyzer(projectRoot string, storageEngine *storage.Engine, logger zerolog.Logger) *ProjectAnalyzer {
	return &ProjectAnalyzer{
		projectRoot:   projectRoot,
		storageEngine: storageEngine,
		logger:        logger.With().Str("component", "analysis").Logger(),
	}
}

// AnalyzeProject performs complete project analysis
func (pa *ProjectAnalyzer) AnalyzeProject() (*AnalysisResult, error) {
	pa.logger.Info().Str("path", pa.projectRoot).Msg("Starting project analysis")

	result := &AnalysisResult{
		AnalyzedAt: time.Now(),
	}

	// Analyze documentation files
	docs, err := pa.analyzeDocumentation()
	if err != nil {
		return nil, fmt.Errorf("failed to analyze documentation: %w", err)
	}
	result.Documentation = docs

	// Analyze project structure and components
	components, err := pa.analyzeProjectStructure()
	if err != nil {
		return nil, fmt.Errorf("failed to analyze project structure: %w", err)
	}
	result.Components = components

	// Extract project metadata
	metadata, err := pa.extractProjectMetadata()
	if err != nil {
		return nil, fmt.Errorf("failed to extract project metadata: %w", err)
	}
	result.Metadata = metadata

	// Store results in database
	if err := pa.storeAnalysisResults(result); err != nil {
		pa.logger.Error().Err(err).Msg("Failed to store analysis results")
		return result, err
	}

	pa.logger.Info().
		Int("docs", len(result.Documentation)).
		Int("components", len(result.Components)).
		Int("metadata", len(result.Metadata)).
		Msg("Project analysis completed")

	return result, nil
}

// analyzeDocumentation finds and analyzes documentation files
func (pa *ProjectAnalyzer) analyzeDocumentation() ([]DocumentationFile, error) {
	var docs []DocumentationFile

	// Common documentation file patterns
	docPatterns := map[string]string{
		"readme":    `^(readme|READ_?ME)(\.(md|txt|rst))?$`,
		"changelog": `^(changelog|CHANGELOG|changes|CHANGES|history|HISTORY)(\.(md|txt|rst))?$`,
		"license":   `^(license|LICENSE|copying|COPYING)(\.(md|txt|rst))?$`,
		"spec":      `^(spec|SPEC|specification|SPECIFICATION)(\.(md|txt|rst))?$`,
		"api":       `^(api|API)(\.(md|txt|rst))?$`,
		"guide":     `^(guide|GUIDE|tutorial|TUTORIAL|howto|HOWTO)(\.(md|txt|rst))?$`,
		"docs":      `^(docs|DOCS|documentation|DOCUMENTATION)(\.(md|txt|rst))?$`,
		"config":    `^(config|CONFIG|configuration|CONFIGURATION)(\.(md|txt|toml|yaml|yml|json))?$`,
		"outline":   `^(outline|OUTLINE)(\.(md|txt|rst))?$`,
		"tasks":     `^(tasks|TASKS|todo|TODO)(\.(md|txt|rst))?$`,
		"status":    `^(status|STATUS|progress|PROGRESS)(\.(md|txt|rst))?$`,
	}

	err := filepath.WalkDir(pa.projectRoot, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}

		// Skip hidden directories and files
		if strings.HasPrefix(d.Name(), ".") {
			if d.IsDir() {
				return fs.SkipDir
			}
			return nil
		}

		// Skip vendor, node_modules, build directories
		if d.IsDir() && (d.Name() == "vendor" || d.Name() == "node_modules" || d.Name() == "build" || d.Name() == "target" || d.Name() == "dist") {
			return fs.SkipDir
		}

		if !d.IsDir() {
			// Check if file matches documentation patterns
			for docType, pattern := range docPatterns {
				matched, _ := regexp.MatchString(pattern, strings.ToLower(d.Name()))
				if matched {
					doc, err := pa.analyzeDocumentationFile(path, docType)
					if err != nil {
						pa.logger.Warn().Err(err).Str("file", path).Msg("Failed to analyze documentation file")
						continue
					}
					docs = append(docs, *doc)
					break
				}
			}
		}

		return nil
	})

	return docs, err
}

// analyzeDocumentationFile analyzes a single documentation file
func (pa *ProjectAnalyzer) analyzeDocumentationFile(filePath, docType string) (*DocumentationFile, error) {
	content, err := os.ReadFile(filePath)
	if err != nil {
		return nil, err
	}

	relPath, _ := filepath.Rel(pa.projectRoot, filePath)
	
	doc := &DocumentationFile{
		FilePath:     relPath,
		DocType:      docType,
		LastAnalyzed: time.Now(),
		ContentHash:  fmt.Sprintf("%x", sha256.Sum256(content)),
	}

	// Extract title and sections from markdown files
	if strings.HasSuffix(strings.ToLower(filePath), ".md") {
		doc.Title, doc.Sections = pa.extractMarkdownStructure(string(content))
		doc.Purpose = pa.inferDocumentPurpose(doc.Title, string(content), docType)
		doc.CrossRefs = pa.extractCrossReferences(string(content))
	} else {
		// For non-markdown files, use filename as title
		doc.Title = filepath.Base(filePath)
		doc.Purpose = pa.inferDocumentPurpose(doc.Title, string(content), docType)
	}

	return doc, nil
}

// extractMarkdownStructure extracts title and section headings from markdown
func (pa *ProjectAnalyzer) extractMarkdownStructure(content string) (string, []string) {
	var title string
	var sections []string

	scanner := bufio.NewScanner(strings.NewReader(content))
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		
		// Extract headings
		if strings.HasPrefix(line, "#") {
			heading := strings.TrimSpace(strings.TrimLeft(line, "#"))
			if heading == "" {
				continue
			}
			
			if title == "" && strings.HasPrefix(line, "# ") {
				title = heading
			} else {
				sections = append(sections, heading)
			}
		}
	}

	return title, sections
}

// inferDocumentPurpose infers the purpose of a document
func (pa *ProjectAnalyzer) inferDocumentPurpose(title, content, docType string) string {
	content = strings.ToLower(content)
	
	switch docType {
	case "readme":
		if strings.Contains(content, "install") || strings.Contains(content, "getting started") {
			return "Project overview and installation guide"
		}
		return "Project introduction and basic documentation"
	case "spec":
		return "Technical specification and architecture details"
	case "api":
		return "API documentation and reference"
	case "guide":
		return "User guide and tutorials"
	case "changelog":
		return "Version history and change log"
	case "license":
		return "Software license and legal information"
	case "config":
		return "Configuration documentation and examples"
	case "outline":
		return "Project outline and feature overview"
	case "tasks":
		return "Task list and implementation tracking"
	case "status":
		return "Project status and progress tracking"
	default:
		return "Documentation file"
	}
}

// extractCrossReferences finds references to other files
func (pa *ProjectAnalyzer) extractCrossReferences(content string) []string {
	var refs []string
	
	// Find markdown links [text](file.md)
	linkRegex := regexp.MustCompile(`\[([^\]]+)\]\(([^)]+\.(md|txt|rst))\)`)
	matches := linkRegex.FindAllStringSubmatch(content, -1)
	for _, match := range matches {
		if len(match) > 2 {
			refs = append(refs, match[2])
		}
	}

	// Find file references without links
	fileRegex := regexp.MustCompile(`\b([A-Z_][A-Z0-9_]*\.(md|txt|rst))\b`)
	matches = fileRegex.FindAllStringSubmatch(content, -1)
	for _, match := range matches {
		if len(match) > 1 {
			refs = append(refs, match[1])
		}
	}

	return refs
}

// analyzeProjectStructure analyzes project structure and identifies components
func (pa *ProjectAnalyzer) analyzeProjectStructure() ([]ProjectComponent, error) {
	var components []ProjectComponent

	// Detect programming language and framework
	langComponents := pa.detectLanguageAndFramework()
	components = append(components, langComponents...)

	// Detect build systems
	buildComponents := pa.detectBuildSystems()
	components = append(components, buildComponents...)

	// Detect project architecture patterns
	archComponents := pa.detectArchitecturePatterns()
	components = append(components, archComponents...)

	return components, nil
}

// detectLanguageAndFramework detects the primary language and frameworks
func (pa *ProjectAnalyzer) detectLanguageAndFramework() []ProjectComponent {
	var components []ProjectComponent

	// Check for Go
	if pa.fileExists("go.mod") {
		goMod := pa.analyzeGoMod()
		components = append(components, goMod...)
	}

	// Check for Node.js
	if pa.fileExists("package.json") {
		nodeComponents := pa.analyzePackageJson()
		components = append(components, nodeComponents...)
	}

	// Check for Python
	if pa.fileExists("requirements.txt") || pa.fileExists("pyproject.toml") || pa.fileExists("setup.py") {
		components = append(components, ProjectComponent{
			Component:   "python",
			Type:        "language",
			Description: "Python programming language",
			Confidence:  0.9,
			Evidence:    pa.findFiles([]string{"requirements.txt", "pyproject.toml", "setup.py"}),
			LastUpdated: time.Now(),
		})
	}

	// Check for Rust
	if pa.fileExists("Cargo.toml") {
		components = append(components, ProjectComponent{
			Component:   "rust",
			Type:        "language",
			Description: "Rust programming language",
			Confidence:  0.9,
			Evidence:    []string{"Cargo.toml"},
			LastUpdated: time.Now(),
		})
	}

	return components
}

// analyzeGoMod analyzes go.mod file for Go project information
func (pa *ProjectAnalyzer) analyzeGoMod() []ProjectComponent {
	var components []ProjectComponent

	goModPath := filepath.Join(pa.projectRoot, "go.mod")
	content, err := os.ReadFile(goModPath)
	if err != nil {
		return components
	}

	goModContent := string(content)
	
	// Extract Go version
	versionRegex := regexp.MustCompile(`go\s+(\d+\.\d+(?:\.\d+)?)`)
	if match := versionRegex.FindStringSubmatch(goModContent); len(match) > 1 {
		components = append(components, ProjectComponent{
			Component:   "go",
			Type:        "language",
			Description: fmt.Sprintf("Go programming language version %s", match[1]),
			Confidence:  1.0,
			Evidence:    []string{"go.mod"},
			Metadata:    map[string]interface{}{"version": match[1]},
			LastUpdated: time.Now(),
		})
	}

	// Detect common frameworks
	if strings.Contains(goModContent, "github.com/charmbracelet/bubbletea") {
		components = append(components, ProjectComponent{
			Component:   "bubbletea",
			Type:        "framework",
			Description: "Charmbracelet Bubble Tea TUI framework",
			Confidence:  0.9,
			Evidence:    []string{"go.mod"},
			LastUpdated: time.Now(),
		})
	}

	if strings.Contains(goModContent, "github.com/gin-gonic/gin") {
		components = append(components, ProjectComponent{
			Component:   "gin",
			Type:        "framework",
			Description: "Gin HTTP web framework",
			Confidence:  0.9,
			Evidence:    []string{"go.mod"},
			LastUpdated: time.Now(),
		})
	}

	return components
}

// analyzePackageJson analyzes package.json for Node.js project information
func (pa *ProjectAnalyzer) analyzePackageJson() []ProjectComponent {
	var components []ProjectComponent

	packageJsonPath := filepath.Join(pa.projectRoot, "package.json")
	content, err := os.ReadFile(packageJsonPath)
	if err != nil {
		return components
	}

	var packageData map[string]interface{}
	if err := json.Unmarshal(content, &packageData); err != nil {
		return components
	}

	components = append(components, ProjectComponent{
		Component:   "nodejs",
		Type:        "language",
		Description: "Node.js JavaScript runtime",
		Confidence:  0.9,
		Evidence:    []string{"package.json"},
		LastUpdated: time.Now(),
	})

	// Check for common frameworks in dependencies
	if deps, ok := packageData["dependencies"].(map[string]interface{}); ok {
		if _, exists := deps["react"]; exists {
			components = append(components, ProjectComponent{
				Component:   "react",
				Type:        "framework",
				Description: "React JavaScript library",
				Confidence:  0.9,
				Evidence:    []string{"package.json"},
				LastUpdated: time.Now(),
			})
		}

		if _, exists := deps["vue"]; exists {
			components = append(components, ProjectComponent{
				Component:   "vue",
				Type:        "framework",
				Description: "Vue.js JavaScript framework",
				Confidence:  0.9,
				Evidence:    []string{"package.json"},
				LastUpdated: time.Now(),
			})
		}
	}

	return components
}

// detectBuildSystems detects build systems and tools
func (pa *ProjectAnalyzer) detectBuildSystems() []ProjectComponent {
	var components []ProjectComponent

	buildFiles := map[string]string{
		"Makefile":       "Make build system",
		"CMakeLists.txt": "CMake build system", 
		"build.gradle":   "Gradle build system",
		"pom.xml":        "Maven build system",
		"docker-compose.yml": "Docker Compose orchestration",
		"Dockerfile":     "Docker containerization",
	}

	for file, description := range buildFiles {
		if pa.fileExists(file) {
			components = append(components, ProjectComponent{
				Component:   strings.ToLower(strings.Replace(file, ".", "_", -1)),
				Type:        "build",
				Description: description,
				Confidence:  0.8,
				Evidence:    []string{file},
				LastUpdated: time.Now(),
			})
		}
	}

	return components
}

// detectArchitecturePatterns detects common architecture patterns
func (pa *ProjectAnalyzer) detectArchitecturePatterns() []ProjectComponent {
	var components []ProjectComponent

	// Check directory structure for common patterns
	dirs := pa.getTopLevelDirectories()

	// MVC pattern
	if pa.containsAll(dirs, []string{"models", "views", "controllers"}) ||
		pa.containsAll(dirs, []string{"model", "view", "controller"}) {
		components = append(components, ProjectComponent{
			Component:   "mvc",
			Type:        "architecture",
			Description: "Model-View-Controller architecture pattern",
			Confidence:  0.7,
			Evidence:    dirs,
			LastUpdated: time.Now(),
		})
	}

	// Clean Architecture / Hexagonal
	if pa.containsAny(dirs, []string{"internal", "pkg", "cmd"}) &&
		pa.containsAny(dirs, []string{"domain", "usecase", "infrastructure"}) {
		components = append(components, ProjectComponent{
			Component:   "clean_architecture",
			Type:        "architecture",
			Description: "Clean Architecture / Hexagonal pattern",
			Confidence:  0.6,
			Evidence:    dirs,
			LastUpdated: time.Now(),
		})
	}

	// Microservices
	if pa.containsAny(dirs, []string{"services", "microservices"}) ||
		pa.fileExists("docker-compose.yml") {
		components = append(components, ProjectComponent{
			Component:   "microservices",
			Type:        "architecture",
			Description: "Microservices architecture",
			Confidence:  0.5,
			Evidence:    append(dirs, "docker-compose.yml"),
			LastUpdated: time.Now(),
		})
	}

	return components
}

// extractProjectMetadata extracts key project metadata
func (pa *ProjectAnalyzer) extractProjectMetadata() ([]ProjectMetadata, error) {
	var metadata []ProjectMetadata

	// Project name from directory
	projectName := filepath.Base(pa.projectRoot)
	metadata = append(metadata, ProjectMetadata{
		Key:         "project_name",
		Value:       projectName,
		Confidence:  0.8,
		Source:      "directory_name",
		Category:    "structure",
		LastUpdated: time.Now(),
	})

	// Extract from go.mod if exists
	if pa.fileExists("go.mod") {
		goMetadata := pa.extractGoModMetadata()
		metadata = append(metadata, goMetadata...)
	}

	// Extract from package.json if exists
	if pa.fileExists("package.json") {
		nodeMetadata := pa.extractPackageJsonMetadata()
		metadata = append(metadata, nodeMetadata...)
	}

	return metadata, nil
}

// extractGoModMetadata extracts metadata from go.mod
func (pa *ProjectAnalyzer) extractGoModMetadata() []ProjectMetadata {
	var metadata []ProjectMetadata

	goModPath := filepath.Join(pa.projectRoot, "go.mod")
	content, err := os.ReadFile(goModPath)
	if err != nil {
		return metadata
	}

	lines := strings.Split(string(content), "\n")
	for _, line := range lines {
		line = strings.TrimSpace(line)
		
		// Module name
		if strings.HasPrefix(line, "module ") {
			moduleName := strings.TrimSpace(strings.TrimPrefix(line, "module"))
			metadata = append(metadata, ProjectMetadata{
				Key:         "go_module",
				Value:       moduleName,
				Confidence:  1.0,
				Source:      "go.mod",
				Category:    "language",
				LastUpdated: time.Now(),
			})
		}

		// Go version
		if strings.HasPrefix(line, "go ") {
			version := strings.TrimSpace(strings.TrimPrefix(line, "go"))
			metadata = append(metadata, ProjectMetadata{
				Key:         "go_version",
				Value:       version,
				Confidence:  1.0,
				Source:      "go.mod",
				Category:    "language",
				LastUpdated: time.Now(),
			})
		}
	}

	return metadata
}

// extractPackageJsonMetadata extracts metadata from package.json
func (pa *ProjectAnalyzer) extractPackageJsonMetadata() []ProjectMetadata {
	var metadata []ProjectMetadata

	packageJsonPath := filepath.Join(pa.projectRoot, "package.json")
	content, err := os.ReadFile(packageJsonPath)
	if err != nil {
		return metadata
	}

	var packageData map[string]interface{}
	if err := json.Unmarshal(content, &packageData); err != nil {
		return metadata
	}

	// Extract key fields
	fields := map[string]string{
		"name":        "project_name",
		"version":     "version", 
		"description": "description",
	}

	for jsonKey, metaKey := range fields {
		if value, ok := packageData[jsonKey].(string); ok {
			metadata = append(metadata, ProjectMetadata{
				Key:         metaKey,
				Value:       value,
				Confidence:  0.9,
				Source:      "package.json",
				Category:    "structure",
				LastUpdated: time.Now(),
			})
		}
	}

	return metadata
}

// Helper functions

func (pa *ProjectAnalyzer) fileExists(filename string) bool {
	_, err := os.Stat(filepath.Join(pa.projectRoot, filename))
	return err == nil
}

func (pa *ProjectAnalyzer) findFiles(filenames []string) []string {
	var found []string
	for _, filename := range filenames {
		if pa.fileExists(filename) {
			found = append(found, filename)
		}
	}
	return found
}

func (pa *ProjectAnalyzer) getTopLevelDirectories() []string {
	var dirs []string
	
	entries, err := os.ReadDir(pa.projectRoot)
	if err != nil {
		return dirs
	}

	for _, entry := range entries {
		if entry.IsDir() && !strings.HasPrefix(entry.Name(), ".") {
			dirs = append(dirs, entry.Name())
		}
	}

	return dirs
}

func (pa *ProjectAnalyzer) containsAll(slice []string, items []string) bool {
	for _, item := range items {
		if !pa.contains(slice, item) {
			return false
		}
	}
	return true
}

func (pa *ProjectAnalyzer) containsAny(slice []string, items []string) bool {
	for _, item := range items {
		if pa.contains(slice, item) {
			return true
		}
	}
	return false
}

func (pa *ProjectAnalyzer) contains(slice []string, item string) bool {
	for _, s := range slice {
		if s == item {
			return true
		}
	}
	return false
}

// storeAnalysisResults stores analysis results in the database
func (pa *ProjectAnalyzer) storeAnalysisResults(result *AnalysisResult) error {
	// This would be implemented to store results in the knowledge database
	// For now, we'll just log that we would store the results
	pa.logger.Info().
		Int("docs_count", len(result.Documentation)).
		Int("components_count", len(result.Components)).
		Int("metadata_count", len(result.Metadata)).
		Msg("Analysis results ready for database storage")
	
	// TODO: Implement actual database storage using pa.storageEngine
	return nil
}