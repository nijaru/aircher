package storage

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/jmoiron/sqlx"
	_ "github.com/mattn/go-sqlite3"
	"github.com/rs/zerolog"
)

// Engine manages all storage operations for Aircher
type Engine struct {
	storageDir      string
	conversationsDB *sqlx.DB
	knowledgeDB     *sqlx.DB
	fileIndexDB     *sqlx.DB
	sessionsDB      *sqlx.DB
	logger          zerolog.Logger
}

// NewEngine creates a new storage engine instance
func NewEngine(storageDir string) (*Engine, error) {
	logger := zerolog.New(os.Stderr).With().Str("component", "storage").Logger()

	// Ensure storage directory exists
	if err := os.MkdirAll(storageDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create storage directory: %w", err)
	}

	// Ensure cache directory exists
	cacheDir := filepath.Join(storageDir, "cache")
	if err := os.MkdirAll(cacheDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create cache directory: %w", err)
	}

	engine := &Engine{
		storageDir: storageDir,
		logger:     logger,
	}

	// Initialize databases
	if err := engine.initializeDatabases(); err != nil {
		return nil, fmt.Errorf("failed to initialize databases: %w", err)
	}

	return engine, nil
}

// initializeDatabases opens and initializes all required databases
func (e *Engine) initializeDatabases() error {
	var err error

	// Initialize conversations database
	e.conversationsDB, err = e.openDatabase("conversations.db")
	if err != nil {
		return fmt.Errorf("failed to open conversations database: %w", err)
	}
	if err := e.initConversationsSchema(); err != nil {
		return fmt.Errorf("failed to initialize conversations schema: %w", err)
	}

	// Initialize knowledge database
	e.knowledgeDB, err = e.openDatabase("knowledge.db")
	if err != nil {
		return fmt.Errorf("failed to open knowledge database: %w", err)
	}
	if err := e.initKnowledgeSchema(); err != nil {
		return fmt.Errorf("failed to initialize knowledge schema: %w", err)
	}

	// Initialize file index database
	e.fileIndexDB, err = e.openDatabase("file_index.db")
	if err != nil {
		return fmt.Errorf("failed to open file index database: %w", err)
	}
	if err := e.initFileIndexSchema(); err != nil {
		return fmt.Errorf("failed to initialize file index schema: %w", err)
	}

	// Initialize sessions database
	e.sessionsDB, err = e.openDatabase("sessions.db")
	if err != nil {
		return fmt.Errorf("failed to open sessions database: %w", err)
	}
	if err := e.initSessionsSchema(); err != nil {
		return fmt.Errorf("failed to initialize sessions schema: %w", err)
	}

	return nil
}

// openDatabase opens a SQLite database with appropriate settings
func (e *Engine) openDatabase(filename string) (*sqlx.DB, error) {
	dbPath := filepath.Join(e.storageDir, filename)
	
	db, err := sqlx.Open("sqlite3", fmt.Sprintf("%s?_journal_mode=WAL&_sync=NORMAL&_cache_size=10000&_temp_store=MEMORY", dbPath))
	if err != nil {
		return nil, err
	}

	// Test connection
	if err := db.Ping(); err != nil {
		db.Close()
		return nil, err
	}

	// Set connection pool settings
	db.SetMaxOpenConns(1) // SQLite works best with single connection
	db.SetMaxIdleConns(1)

	return db, nil
}

// initConversationsSchema creates tables for conversation storage
func (e *Engine) initConversationsSchema() error {
	schema := `
	CREATE TABLE IF NOT EXISTS conversations (
		id TEXT PRIMARY KEY,
		created_at INTEGER NOT NULL,
		updated_at INTEGER NOT NULL,
		title TEXT,
		provider TEXT NOT NULL,
		model TEXT NOT NULL,
		total_tokens INTEGER DEFAULT 0,
		total_cost REAL DEFAULT 0.0,
		message_count INTEGER DEFAULT 0
	);

	CREATE TABLE IF NOT EXISTS messages (
		id TEXT PRIMARY KEY,
		conversation_id TEXT NOT NULL,
		role TEXT NOT NULL CHECK (role IN ('system', 'user', 'assistant', 'tool')),
		content TEXT NOT NULL,
		tokens INTEGER DEFAULT 0,
		cost REAL DEFAULT 0.0,
		created_at INTEGER NOT NULL,
		tool_calls TEXT, -- JSON array of tool calls
		tool_results TEXT, -- JSON array of tool results
		FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
	);

	CREATE TABLE IF NOT EXISTS message_files (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		message_id TEXT NOT NULL,
		file_path TEXT NOT NULL,
		relevance_score REAL DEFAULT 0.0,
		included_reason TEXT,
		FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
	);

	CREATE INDEX IF NOT EXISTS idx_conversations_updated_at ON conversations(updated_at);
	CREATE INDEX IF NOT EXISTS idx_messages_conversation_id ON messages(conversation_id);
	CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);
	CREATE INDEX IF NOT EXISTS idx_message_files_message_id ON message_files(message_id);
	CREATE INDEX IF NOT EXISTS idx_message_files_file_path ON message_files(file_path);
	`

	_, err := e.conversationsDB.Exec(schema)
	return err
}

// initKnowledgeSchema creates tables for project knowledge storage
func (e *Engine) initKnowledgeSchema() error {
	schema := `
	CREATE TABLE IF NOT EXISTS decisions (
		id TEXT PRIMARY KEY,
		title TEXT NOT NULL,
		description TEXT NOT NULL,
		reasoning TEXT,
		alternatives TEXT, -- JSON array
		created_at INTEGER NOT NULL,
		updated_at INTEGER NOT NULL,
		tags TEXT, -- JSON array
		status TEXT DEFAULT 'active' CHECK (status IN ('active', 'deprecated', 'superseded'))
	);

	CREATE TABLE IF NOT EXISTS patterns (
		id TEXT PRIMARY KEY,
		pattern_type TEXT NOT NULL,
		pattern TEXT NOT NULL,
		description TEXT,
		confidence REAL DEFAULT 0.0,
		usage_count INTEGER DEFAULT 0,
		last_seen INTEGER NOT NULL,
		examples TEXT, -- JSON array
		tags TEXT -- JSON array
	);

	CREATE TABLE IF NOT EXISTS code_insights (
		id TEXT PRIMARY KEY,
		file_path TEXT NOT NULL,
		insight_type TEXT NOT NULL,
		insight TEXT NOT NULL,
		confidence REAL DEFAULT 0.0,
		created_at INTEGER NOT NULL,
		updated_at INTEGER NOT NULL,
		metadata TEXT -- JSON object
	);

	CREATE TABLE IF NOT EXISTS documentation_analysis (
		file_path TEXT PRIMARY KEY,
		doc_type TEXT NOT NULL, -- 'readme', 'spec', 'guide', 'api', 'config'
		title TEXT,
		purpose TEXT,
		sections TEXT, -- JSON array of section headings
		cross_refs TEXT, -- JSON array of references to other docs
		last_analyzed INTEGER NOT NULL,
		content_hash TEXT -- for change detection
	);

	CREATE TABLE IF NOT EXISTS project_structure_analysis (
		component TEXT PRIMARY KEY,
		type TEXT NOT NULL, -- 'framework', 'architecture', 'convention', 'dependency'
		description TEXT NOT NULL,
		confidence REAL DEFAULT 0.0, -- 0.0 to 1.0
		evidence TEXT, -- JSON array of supporting files/patterns
		metadata TEXT, -- JSON object with additional info
		last_updated INTEGER NOT NULL
	);

	CREATE TABLE IF NOT EXISTS project_metadata (
		key TEXT PRIMARY KEY,
		value TEXT NOT NULL,
		confidence REAL DEFAULT 0.0,
		source TEXT, -- where this info came from
		category TEXT, -- 'language', 'framework', 'build', 'structure'
		last_updated INTEGER NOT NULL
	);

	CREATE INDEX IF NOT EXISTS idx_decisions_created_at ON decisions(created_at);
	CREATE INDEX IF NOT EXISTS idx_decisions_status ON decisions(status);
	CREATE INDEX IF NOT EXISTS idx_patterns_type ON patterns(pattern_type);
	CREATE INDEX IF NOT EXISTS idx_patterns_last_seen ON patterns(last_seen);
	CREATE INDEX IF NOT EXISTS idx_code_insights_file_path ON code_insights(file_path);
	CREATE INDEX IF NOT EXISTS idx_code_insights_type ON code_insights(insight_type);
	CREATE INDEX IF NOT EXISTS idx_documentation_analysis_doc_type ON documentation_analysis(doc_type);
	CREATE INDEX IF NOT EXISTS idx_project_structure_type ON project_structure_analysis(type);
	CREATE INDEX IF NOT EXISTS idx_project_metadata_category ON project_metadata(category);
	`

	_, err := e.knowledgeDB.Exec(schema)
	return err
}

// initFileIndexSchema creates tables for file relationship tracking
func (e *Engine) initFileIndexSchema() error {
	schema := `
	CREATE TABLE IF NOT EXISTS files (
		path TEXT PRIMARY KEY,
		size INTEGER NOT NULL,
		modified_at INTEGER NOT NULL,
		file_type TEXT,
		language TEXT,
		checksum TEXT,
		indexed_at INTEGER NOT NULL,
		access_count INTEGER DEFAULT 0,
		last_accessed INTEGER
	);

	CREATE TABLE IF NOT EXISTS dependencies (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		source_file TEXT NOT NULL,
		target_file TEXT NOT NULL,
		dependency_type TEXT NOT NULL,
		strength REAL DEFAULT 1.0,
		detected_at INTEGER NOT NULL,
		FOREIGN KEY (source_file) REFERENCES files(path) ON DELETE CASCADE,
		FOREIGN KEY (target_file) REFERENCES files(path) ON DELETE CASCADE,
		UNIQUE(source_file, target_file, dependency_type)
	);

	CREATE TABLE IF NOT EXISTS file_changes (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		file_path TEXT NOT NULL,
		change_type TEXT NOT NULL CHECK (change_type IN ('created', 'modified', 'deleted', 'renamed')),
		old_path TEXT,
		detected_at INTEGER NOT NULL,
		FOREIGN KEY (file_path) REFERENCES files(path) ON DELETE CASCADE
	);

	CREATE TABLE IF NOT EXISTS relevance_cache (
		file_path TEXT PRIMARY KEY,
		task_context TEXT NOT NULL,
		relevance_score REAL NOT NULL,
		calculated_at INTEGER NOT NULL,
		expires_at INTEGER NOT NULL,
		factors TEXT, -- JSON object describing why this score was assigned
		FOREIGN KEY (file_path) REFERENCES files(path) ON DELETE CASCADE
	);

	CREATE INDEX IF NOT EXISTS idx_files_modified_at ON files(modified_at);
	CREATE INDEX IF NOT EXISTS idx_files_language ON files(language);
	CREATE INDEX IF NOT EXISTS idx_files_last_accessed ON files(last_accessed);
	CREATE INDEX IF NOT EXISTS idx_dependencies_source ON dependencies(source_file);
	CREATE INDEX IF NOT EXISTS idx_dependencies_target ON dependencies(target_file);
	CREATE INDEX IF NOT EXISTS idx_file_changes_detected_at ON file_changes(detected_at);
	CREATE INDEX IF NOT EXISTS idx_relevance_cache_expires_at ON relevance_cache(expires_at);
	`

	_, err := e.fileIndexDB.Exec(schema)
	return err
}

// initSessionsSchema creates tables for session management
func (e *Engine) initSessionsSchema() error {
	schema := `
	CREATE TABLE IF NOT EXISTS sessions (
		id TEXT PRIMARY KEY,
		created_at INTEGER NOT NULL,
		updated_at INTEGER NOT NULL,
		conversation_id TEXT,
		provider TEXT,
		model TEXT,
		context_snapshot TEXT, -- JSON object
		status TEXT DEFAULT 'active' CHECK (status IN ('active', 'paused', 'ended')),
		FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE SET NULL
	);

	CREATE TABLE IF NOT EXISTS session_context (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		session_id TEXT NOT NULL,
		context_type TEXT NOT NULL,
		context_data TEXT NOT NULL, -- JSON data
		priority INTEGER DEFAULT 0,
		created_at INTEGER NOT NULL,
		FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
	);

	CREATE INDEX IF NOT EXISTS idx_sessions_updated_at ON sessions(updated_at);
	CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
	CREATE INDEX IF NOT EXISTS idx_session_context_session_id ON session_context(session_id);
	CREATE INDEX IF NOT EXISTS idx_session_context_type ON session_context(context_type);
	`

	_, err := e.sessionsDB.Exec(schema)
	return err
}

// GetConversationsDB returns the conversations database connection
func (e *Engine) GetConversationsDB() *sqlx.DB {
	return e.conversationsDB
}

// GetKnowledgeDB returns the knowledge database connection
func (e *Engine) GetKnowledgeDB() *sqlx.DB {
	return e.knowledgeDB
}

// GetFileIndexDB returns the file index database connection
func (e *Engine) GetFileIndexDB() *sqlx.DB {
	return e.fileIndexDB
}

// GetSessionsDB returns the sessions database connection
func (e *Engine) GetSessionsDB() *sqlx.DB {
	return e.sessionsDB
}

// GetStorageDir returns the storage directory path
func (e *Engine) GetStorageDir() string {
	return e.storageDir
}

// GetCacheDir returns the cache directory path
func (e *Engine) GetCacheDir() string {
	return filepath.Join(e.storageDir, "cache")
}

// Close closes all database connections
func (e *Engine) Close() error {
	var errors []error

	if e.conversationsDB != nil {
		if err := e.conversationsDB.Close(); err != nil {
			errors = append(errors, fmt.Errorf("failed to close conversations DB: %w", err))
		}
	}

	if e.knowledgeDB != nil {
		if err := e.knowledgeDB.Close(); err != nil {
			errors = append(errors, fmt.Errorf("failed to close knowledge DB: %w", err))
		}
	}

	if e.fileIndexDB != nil {
		if err := e.fileIndexDB.Close(); err != nil {
			errors = append(errors, fmt.Errorf("failed to close file index DB: %w", err))
		}
	}

	if e.sessionsDB != nil {
		if err := e.sessionsDB.Close(); err != nil {
			errors = append(errors, fmt.Errorf("failed to close sessions DB: %w", err))
		}
	}

	if len(errors) > 0 {
		return fmt.Errorf("storage close errors: %v", errors)
	}

	return nil
}

// RunMigrations runs any pending database migrations
func (e *Engine) RunMigrations() error {
	// For now, just re-run schema initialization to handle any new tables/indexes
	if err := e.initConversationsSchema(); err != nil {
		return fmt.Errorf("failed to migrate conversations schema: %w", err)
	}

	if err := e.initKnowledgeSchema(); err != nil {
		return fmt.Errorf("failed to migrate knowledge schema: %w", err)
	}

	if err := e.initFileIndexSchema(); err != nil {
		return fmt.Errorf("failed to migrate file index schema: %w", err)
	}

	if err := e.initSessionsSchema(); err != nil {
		return fmt.Errorf("failed to migrate sessions schema: %w", err)
	}

	return nil
}

// HealthCheck performs basic health checks on all databases
func (e *Engine) HealthCheck() error {
	databases := map[string]*sqlx.DB{
		"conversations": e.conversationsDB,
		"knowledge":     e.knowledgeDB,
		"file_index":    e.fileIndexDB,
		"sessions":      e.sessionsDB,
	}

	for name, db := range databases {
		if db == nil {
			return fmt.Errorf("database %s is not initialized", name)
		}

		if err := db.Ping(); err != nil {
			return fmt.Errorf("database %s health check failed: %w", name, err)
		}

		// Test basic query
		var result int
		if err := db.Get(&result, "SELECT 1"); err != nil {
			return fmt.Errorf("database %s query test failed: %w", name, err)
		}
	}

	return nil
}