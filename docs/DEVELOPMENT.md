# Aircher Development Guide

## Recent Achievements 🎉

### Major Cleanup Milestone (Latest)
- ✅ **Zero Compiler Warnings**: Reduced from ~190 warnings to 0
- ✅ **Query Expansion**: Added synonym expansion and typo correction
- ✅ **HuggingFace Integration**: Complete model download system
- ✅ **License Compliance**: Replaced SweRankEmbed with Apache/MIT models
- ✅ **Code Quality**: Removed unused code, fixed field access patterns

### Performance Achievements
- 🚀 **99.9% Speed Improvement**: Instant subsequent searches via index persistence
- ⚡ **Production Ready**: Optimized for real-world usage patterns

## Setup

### Prerequisites
```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Development tools
cargo install cargo-watch cargo-audit
```

### Project Setup
```bash
git clone https://github.com/username/aircher
cd aircher

# Build application
cargo build

# Run tests
cargo test --all
```

## Project Structure

### Repository Organization
```
aircher/
├── src/                    # Rust source code
├── docs/                   # Documentation (CORE.md, ARCHITECTURE.md, etc.)
├── examples/               # Configuration examples and demos
├── external/               # Reference implementations (tracked in git)
│   ├── codex/             # Codex CLI and TUI examples
│   ├── oli/               # Oli terminal AI assistant
│   ├── gemini-cli/        # Google AI CLI reference
│   └── opencode/          # OpenCode terminal tools
├── private/                # Local development files (gitignored)
├── Cargo.toml             # Rust project configuration
├── AGENT.md               # AI agent instructions and workflow (universal compatibility)
└── README.md              # Project overview
```

### Development References
The `external/` directory contains reference implementations that are tracked in git and useful for development:

- **Codex**: TUI patterns, command popup, modal overlays
- **Oli**: Model selection UI, welcome screens
- **Gemini CLI**: Configuration patterns, provider integration
- **OpenCode**: Selection interfaces, provider switching

These references help understand proven patterns but aren't part of our codebase.

## Development Workflow

### Core Commands
```bash
# Development build with watching
cargo watch -x build

# Test everything
cargo test --all

# Linting and formatting
cargo clippy --all-targets --all-features
cargo fmt

# Release build
cargo build --release
```

### Pre-commit Checklist
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo test` passes all tests
- [ ] `cargo fmt` has been run
- [ ] Integration tests pass
- [ ] Update task status in `docs/tasks/tasks.json`

## Code Standards

### Rust Conventions
```rust
// Naming: snake_case for functions, PascalCase for types
pub fn analyze_project(files: &[PathBuf]) -> Result<ProjectContext> { }
pub struct ProviderManager { }

// Error handling: Use Result types, custom error enums
#[derive(Debug, thiserror::Error)]
pub enum ArcherError {
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),
}

// Async: Use tokio runtime, prefer async/await
#[tokio::main]
async fn main() -> Result<()> { }

// Traits: Interface-based design
pub trait LLMProvider {
    fn name(&self) -> &str;
    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse>;
}
```

### Intelligence Engine Standards
```rust
// Intelligence components use async/await patterns
pub struct ProjectAnalyzer {
    git_repo: git2::Repository,
    parsers: HashMap<String, tree_sitter::Parser>,
}

impl ProjectAnalyzer {
    pub async fn analyze_structure(&self, files: &[PathBuf]) -> Result<ProjectStructure> {
        // Use tree-sitter for AST parsing
        // Use git2 for git status analysis
        // Return structured analysis results
    }
}
```

### Configuration Patterns
```toml
# Use TOML for all configuration
[providers.openai]
api_key_env = "OPENAI_API_KEY"  # Environment variable names
default_model = "gpt-4o"        # Default values
timeout_seconds = 30            # Explicit units
```

## Architecture Patterns

### Provider Pattern
```rust
// All LLM providers implement common interface
pub trait LLMProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse>;
    async fn stream(&self, req: &ChatRequest) -> Result<ResponseStream>;
    fn supports_feature(&self, feature: Feature) -> bool;
}

// Register providers in manager
pub struct ProviderManager {
    providers: HashMap<String, Box<dyn LLMProvider>>,
}

impl ProviderManager {
    pub fn register<P: LLMProvider + 'static>(&mut self, provider: P) {
        self.providers.insert(provider.name().to_string(), Box::new(provider));
    }
}
```

### Database Pattern  
```rust
// Separate concerns with dedicated databases
pub trait DatabaseConnection {
    async fn execute(&self, query: &str) -> Result<()>;
    async fn fetch_one<T>(&self, query: &str) -> Result<T> where T: FromRow;
}

// Use connection pools
pub struct DatabaseManager {
    conversations: Pool<Sqlite>,
    knowledge: Pool<Sqlite>,
    file_index: Pool<Sqlite>,
    sessions: Pool<Sqlite>,
}
```

### Configuration Pattern
```rust
// Hierarchical config with serde
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub global: GlobalConfig,
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(default)]
    pub ui: UiConfig,
}

// Environment variable resolution
impl Config {
    pub fn load() -> Result<Self> {
        let config_str = fs::read_to_string("config.toml")?;
        let mut config: Config = toml::from_str(&config_str)?;
        config.resolve_env_vars()?;
        Ok(config)
    }
}
```

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_provider_chat() {
        let provider = MockProvider::new();
        let req = ChatRequest::new("Hello");
        let resp = provider.chat(&req).await.unwrap();
        assert!(!resp.content.is_empty());
    }
}
```

### Integration Tests
```rust
// tests/integration/provider_tests.rs
#[tokio::test]
async fn test_openai_provider_integration() {
    if env::var("OPENAI_API_KEY").is_err() {
        return; // Skip if no API key
    }
    
    let provider = OpenAIProvider::new(&config);
    let req = ChatRequest::simple("Hello");
    let resp = provider.chat(&req).await.unwrap();
    assert!(resp.content.len() > 0);
}
```

### Intelligence Engine Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_project_analysis() {
        let analyzer = ProjectAnalyzer::new("test_repo").unwrap();
        let files = vec![PathBuf::from("src/main.rs")];
        let result = analyzer.analyze_structure(&files).await.unwrap();
        assert!(!result.files.is_empty());
    }
    
    #[tokio::test]  
    async fn test_file_scoring() {
        let scorer = FileRelevanceScorer::new();
        let git_status = GitStatus::new();
        let scores = scorer.score_files(&[PathBuf::from("test.rs")], &git_status).await.unwrap();
        assert!(scores[0].relevance >= 0.0 && scores[0].relevance <= 1.0);
    }
}
```

## Performance Guidelines

### Rust Performance
```rust
// Prefer string slices over owned strings
fn process_text(text: &str) -> &str { text }

// Use async for I/O, sync for CPU-bound work
async fn fetch_data() -> Result<String> { }
fn compute_hash(data: &[u8]) -> u64 { }

// Minimize allocations in hot paths
fn efficient_processing(items: &[Item]) -> Vec<Result> {
    items.iter()
         .map(|item| process_item(item))
         .collect()
}
```

### Intelligence Engine Performance
```rust
// Use efficient data structures and algorithms
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct FileCache {
    cache: RwLock<HashMap<PathBuf, CachedAnalysis>>,
}

impl FileCache {
    pub async fn get_or_analyze(&self, file: &PathBuf) -> Result<Analysis> {
        // Check cache first
        if let Some(cached) = self.cache.read().await.get(file) {
            if !cached.is_stale() {
                return Ok(cached.analysis.clone());
            }
        }
        
        // Analyze and cache
        let analysis = self.analyze_file(file).await?;
        self.cache.write().await.insert(file.clone(), CachedAnalysis::new(analysis.clone()));
        Ok(analysis)
    }
}
```

## Debugging

### Rust Debugging
```rust
// Use tracing for structured logging
use tracing::{info, warn, error, debug};

#[tracing::instrument]
async fn process_request(req: &ChatRequest) -> Result<ChatResponse> {
    debug!("Processing request with {} messages", req.messages.len());
    
    let start = Instant::now();
    let response = provider.chat(req).await?;
    
    info!(
        duration_ms = start.elapsed().as_millis(),
        tokens = response.tokens_used,
        "Request completed"
    );
    
    Ok(response)
}

// Initialize tracing
tracing_subscriber::fmt()
    .with_env_filter("aircher=debug")
    .init();
```

### Intelligence Engine Debugging
```rust
use tracing::{info, debug, error, instrument};

impl IntelligenceEngine {
    #[instrument(skip(self, files))]
    pub async fn analyze_project(&self, files: &[PathBuf]) -> Result<ProjectContext> {
        debug!("Starting project analysis for {} files", files.len());
        
        let git_status = self.project_analyzer.get_git_status().await?;
        debug!("Git status analysis complete: {} modified files", git_status.modified_count());
        
        let file_scores = self.file_scorer.score_files(files, &git_status).await?;
        info!("File scoring complete, average relevance: {:.2}", 
              file_scores.iter().map(|s| s.relevance).sum::<f64>() / files.len() as f64);
        
        Ok(ProjectContext { git_status, files: file_scores, /* ... */ })
    }
}
```

## Task Management for AI Agents

### Task Status Updates
```bash
# View current tasks
jq '.tasks | to_entries | map(select(.value.status == "pending"))' docs/tasks/tasks.json

# Mark task in progress
jq '.tasks["TASK-001"].status = "in_progress"' docs/tasks/tasks.json > tmp.json && mv tmp.json docs/tasks/tasks.json

# Complete task
jq '.tasks["TASK-001"].status = "completed"' docs/tasks/tasks.json > tmp.json && mv tmp.json docs/tasks/tasks.json

# Add new task
jq '.tasks["TASK-NEW"] = {
    "description": "Task description",
    "status": "pending", 
    "priority": "high",
    "files": ["src/main.rs"],
    "acceptance_criteria": ["Criteria 1", "Criteria 2"]
}' docs/tasks/tasks.json > tmp.json && mv tmp.json docs/tasks/tasks.json
```

### Development Session Workflow
1. **Start**: Check `docs/tasks/tasks.json` for pending tasks
2. **Plan**: Mark task as "in_progress" and review acceptance criteria
3. **Implement**: Follow coding standards and architectural patterns
4. **Test**: Run full test suite and linting
5. **Complete**: Mark task as "completed" and update documentation
6. **Commit**: Clean commit messages without AI attribution

## Common Patterns

### Error Recovery
```rust
// Graceful fallbacks for provider failures
async fn chat_with_fallback(req: &ChatRequest) -> Result<ChatResponse> {
    match primary_provider.chat(req).await {
        Ok(response) => Ok(response),
        Err(e) => {
            warn!("Primary provider failed: {}, trying fallback", e);
            fallback_provider.chat(req).await
        }
    }
}
```

### Configuration Loading
```rust
// Environment-aware configuration
impl Config {
    pub fn load_for_environment() -> Result<Self> {
        let env = env::var("AIRCHER_ENV").unwrap_or_else(|_| "development".to_string());
        
        let config_path = match env.as_str() {
            "production" => "config/production.toml",
            "test" => "config/test.toml", 
            _ => "config/development.toml",
        };
        
        Self::load_from_file(config_path)
    }
}
```

### UI State Management
```rust
// Clean separation of UI state and business logic
pub struct AppState {
    current_conversation: Option<ConversationId>,
    selected_provider: String,
    selected_model: String,
    ui_mode: UiMode,
}

impl AppState {
    pub fn switch_provider(&mut self, provider: String) -> Result<()> {
        // Validate provider exists
        if !self.provider_manager.has_provider(&provider) {
            return Err(ArcherError::InvalidProvider(provider));
        }
        
        self.selected_provider = provider;
        Ok(())
    }
}
```

This development guide provides the essential patterns and workflows for maintaining code quality and architectural consistency in Aircher.