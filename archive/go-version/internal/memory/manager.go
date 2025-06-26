package memory

import (
	"fmt"
	"path/filepath"
	"time"

	"github.com/aircher/aircher/internal/analysis"
	"github.com/aircher/aircher/internal/config"
	"github.com/aircher/aircher/internal/storage"
	"github.com/rs/zerolog"
)

// Manager handles project memory and AGENTS.md integration
type Manager struct {
	config          *config.Config
	projectRoot     string
	storageEngine   *storage.Engine
	logger          zerolog.Logger
	projectMemory   *ProjectMemoryFile
	userMemory      *ProjectMemoryFile
	projectAnalyzer *analysis.ProjectAnalyzer
	docGenerator    *analysis.DocumentationGenerator
	lastAnalyzedAt  time.Time
}

// ProjectMemoryFile represents an AGENTS.md memory file
type ProjectMemoryFile struct {
	FilePath     string
	Instructions []string
	Conventions  []string
	Commands     map[string]string
	Architecture []string
	Glossary     map[string]string
	Dependencies []string
	LastModified int64
	SyncedToDB   bool
}

// NewManager creates a new memory manager
func NewManager(cfg *config.Config, projectRoot string, storageEngine *storage.Engine, logger zerolog.Logger) (*Manager, error) {
	memoryLogger := logger.With().Str("component", "memory").Logger()

	// Create project analyzer and documentation generator
	projectAnalyzer := analysis.NewProjectAnalyzer(projectRoot, storageEngine, memoryLogger)
	docGenerator := analysis.NewDocumentationGenerator(projectRoot, memoryLogger)

	return &Manager{
		config:          cfg,
		projectRoot:     projectRoot,
		storageEngine:   storageEngine,
		logger:          memoryLogger,
		projectAnalyzer: projectAnalyzer,
		docGenerator:    docGenerator,
	}, nil
}

// LoadProjectMemory loads project memory from AGENTS.md and triggers analysis
func (m *Manager) LoadProjectMemory() error {
	m.logger.Info().Msg("Loading project memory")

	memoryPath := filepath.Join(m.projectRoot, m.config.Memory.ProjectFile)

	// TODO: Implement AGENTS.md parsing
	m.logger.Debug().Str("path", memoryPath).Msg("Memory loading not yet implemented")

	// Trigger automatic project analysis
	if err := m.analyzeAndGenerateDocumentation(); err != nil {
		m.logger.Warn().Err(err).Msg("Failed to analyze project automatically")
		// Don't fail the entire load process if analysis fails
	}

	return nil
}

// InitializeProject creates initial AGENTS.md file
func (m *Manager) InitializeProject() error {
	m.logger.Info().Msg("Initializing project memory")

	memoryPath := filepath.Join(m.projectRoot, m.config.Memory.ProjectFile)

	// TODO: Implement AGENTS.md creation
	m.logger.Debug().Str("path", memoryPath).Msg("Project initialization not yet implemented")

	return fmt.Errorf("project initialization not yet implemented")
}

// HandleMemoryInput processes memory-related input
func (m *Manager) HandleMemoryInput(input string) error {
	// TODO: Implement memory input handling
	return fmt.Errorf("memory input handling not yet implemented")
}

// SyncToDatabase synchronizes memory to database
func (m *Manager) SyncToDatabase() error {
	// TODO: Implement database synchronization
	return nil
}

// GetProjectMemory returns current project memory
func (m *Manager) GetProjectMemory() *ProjectMemoryFile {
	return m.projectMemory
}

// analyzeAndGenerateDocumentation performs automatic project analysis
func (m *Manager) analyzeAndGenerateDocumentation() error {
	m.logger.Info().Msg("Starting automatic project analysis")

	// Perform project analysis
	result, err := m.projectAnalyzer.AnalyzeProject()
	if err != nil {
		return fmt.Errorf("project analysis failed: %w", err)
	}

	// Generate documentation
	if err := m.docGenerator.GenerateProjectAnalysis(result); err != nil {
		return fmt.Errorf("documentation generation failed: %w", err)
	}

	m.lastAnalyzedAt = time.Now()
	m.logger.Info().Msg("Automatic project analysis completed")

	return nil
}

// RefreshProjectAnalysis triggers a new analysis and documentation generation
func (m *Manager) RefreshProjectAnalysis() error {
	return m.analyzeAndGenerateDocumentation()
}

// GetProjectAnalysisPath returns the path to the auto-generated project analysis
func (m *Manager) GetProjectAnalysisPath() string {
	return filepath.Join(m.projectRoot, ".aircher", "project_analysis.md")
}

// ShouldRefreshAnalysis determines if analysis should be refreshed based on time
func (m *Manager) ShouldRefreshAnalysis(maxAge time.Duration) bool {
	if m.lastAnalyzedAt.IsZero() {
		return true
	}
	return time.Since(m.lastAnalyzedAt) > maxAge
}
