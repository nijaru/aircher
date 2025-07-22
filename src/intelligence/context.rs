use crate::config::ConfigManager;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use git2::Repository;
use super::{FileWithContext, ImpactAnalysis, ContextSuggestions, ContextualRelevance, Action};
use walkdir::WalkDir;
use std::time::SystemTime;
use std::sync::{Arc, Mutex};

/// Information about a file in the project
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
    pub file_type: String,
    pub git_status: Option<String>,
    pub lines_of_code: usize,
}

/// Contextual Relevance Engine - Multi-layered file relevance scoring
pub struct ContextualRelevanceEngine {
    _config: ConfigManager,
    project_root: PathBuf,
    git_repo: Arc<Mutex<Option<Repository>>>,
    file_cache: HashMap<String, FileInfo>,
}

impl ContextualRelevanceEngine {
    pub async fn new(config: &ConfigManager) -> Result<Self> {
        // Determine project root (current working directory)
        let project_root = std::env::current_dir()?;
        
        // Try to open git repository
        let git_repo = Arc::new(Mutex::new(Repository::open(&project_root).ok()));
        
        let mut engine = Self {
            _config: config.clone(),
            project_root,
            git_repo,
            file_cache: HashMap::new(),
        };
        
        // Initialize file cache
        engine.scan_project_files().await?;
        
        Ok(engine)
    }

    /// Scan project files and build cache
    async fn scan_project_files(&mut self) -> Result<()> {
        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let relative_path = path.strip_prefix(&self.project_root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
            
            // Skip binary files and common ignore patterns
            if self.should_skip_file(&relative_path) {
                continue;
            }
            
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            
            let file_info = FileInfo {
                path: path.to_path_buf(),
                size: metadata.len(),
                modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                file_type: self.get_file_type(&relative_path),
                git_status: self.get_git_status(&relative_path),
                lines_of_code: self.count_lines_of_code(path).unwrap_or(0),
            };
            
            self.file_cache.insert(relative_path, file_info);
        }
        
        Ok(())
    }
    
    /// Check if file should be skipped during scanning
    fn should_skip_file(&self, path: &str) -> bool {
        let skip_patterns = [
            ".git/", "target/", "node_modules/", ".DS_Store",
            ".exe", ".dll", ".so", ".dylib", ".png", ".jpg", ".jpeg", ".gif",
            ".pdf", ".zip", ".tar", ".gz", ".bz2", ".xz"
        ];
        
        skip_patterns.iter().any(|pattern| path.contains(pattern))
    }
    
    /// Get file type based on extension
    fn get_file_type(&self, path: &str) -> String {
        Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_string()
    }
    
    /// Get git status for file
    fn get_git_status(&self, path: &str) -> Option<String> {
        if let Ok(repo_guard) = self.git_repo.lock() {
            if let Some(repo) = repo_guard.as_ref() {
                let status = repo.status_file(Path::new(path)).ok()?;
                if status.is_wt_modified() {
                    Some("modified".to_string())
                } else if status.is_wt_new() {
                    Some("new".to_string())
                } else if status.is_index_modified() {
                    Some("staged".to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Count lines of code in a file
    fn count_lines_of_code(&self, path: &Path) -> Result<usize> {
        let content = fs::read_to_string(path)?;
        Ok(content.lines().count())
    }
    
    /// Analyze relevance of files for a given query/context
    pub async fn analyze_relevance(&self, query: &str) -> RelevanceAnalysis {
        let mut ranked_files = Vec::new();
        
        // Get recently modified files (immediate relevance)
        let _recent_threshold = SystemTime::now() - std::time::Duration::from_secs(24 * 60 * 60); // 24 hours
        
        for (path, file_info) in &self.file_cache {
            let relevance = self.calculate_file_relevance(path, file_info, query);
            
            // Only include files with some relevance
            if relevance.total_score() > 0.1 {
                ranked_files.push(FileWithContext {
                    path: path.clone(),
                    relevance,
                    purpose: self.determine_file_purpose(path, file_info),
                    last_significant_change: file_info.modified.into(),
                    relationship_to_current_work: self.analyze_relationship(path, query),
                });
            }
        }
        
        // Sort by total relevance score
        ranked_files.sort_by(|a, b| b.relevance.total_score().partial_cmp(&a.relevance.total_score()).unwrap());
        
        // Take top 20 files
        ranked_files.truncate(20);
        
        let confidence = if ranked_files.is_empty() {
            0.0
        } else {
            ranked_files.iter().map(|f| f.relevance.total_score()).sum::<f64>() / ranked_files.len() as f64
        };
        
        RelevanceAnalysis {
            ranked_files,
            confidence,
            predicted_actions: self.predict_next_actions(query),
        }
    }
    
    /// Calculate multi-layered relevance score for a file
    fn calculate_file_relevance(&self, path: &str, file_info: &FileInfo, query: &str) -> ContextualRelevance {
        let mut relevance = ContextualRelevance {
            immediate: 0.0,
            sequential: 0.0,
            dependent: 0.0,
            reference: 0.0,
            historical: 0.0,
        };
        
        // Immediate relevance: recently modified files
        if file_info.git_status.is_some() {
            relevance.immediate += 0.8;
        }
        
        let hours_since_modified = file_info.modified
            .elapsed()
            .unwrap_or_default()
            .as_secs() as f64 / 3600.0;
        
        if hours_since_modified < 24.0 {
            relevance.immediate += 0.6 * (1.0 - hours_since_modified / 24.0);
        }
        
        // Sequential relevance: files that match query context
        if self.matches_query_context(path, query) {
            relevance.sequential += 0.7;
        }
        
        // Dependent relevance: important architectural files
        if self.is_architectural_file(path) {
            relevance.dependent += 0.5;
        }
        
        // Reference relevance: configuration and documentation
        if self.is_reference_file(path) {
            relevance.reference += 0.4;
        }
        
        // Historical relevance: files with significant git history
        if let Ok(repo_guard) = self.git_repo.lock() {
            if let Some(repo) = repo_guard.as_ref() {
                if let Ok(commit_count) = self.count_recent_commits(repo, path) {
                    relevance.historical += (commit_count as f64 * 0.1).min(0.5);
                }
            }
        }
        
        relevance
    }
    
    /// Check if file matches query context
    fn matches_query_context(&self, path: &str, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        let path_lower = path.to_lowercase();
        
        // Simple keyword matching
        query_lower.split_whitespace().any(|word| {
            path_lower.contains(word) || 
            self.get_file_keywords(path).iter().any(|keyword| keyword.contains(word))
        })
    }
    
    /// Get keywords associated with a file
    fn get_file_keywords(&self, path: &str) -> Vec<String> {
        let mut keywords = Vec::new();
        
        // Add file name components
        if let Some(file_name) = Path::new(path).file_stem() {
            keywords.push(file_name.to_string_lossy().to_string());
        }
        
        // Add directory components
        for component in Path::new(path).components() {
            if let Some(name) = component.as_os_str().to_str() {
                keywords.push(name.to_string());
            }
        }
        
        keywords
    }
    
    /// Check if file is architectural (core system files)
    fn is_architectural_file(&self, path: &str) -> bool {
        let arch_patterns = ["mod.rs", "lib.rs", "main.rs", "config", "provider", "core", "engine"];
        arch_patterns.iter().any(|pattern| path.contains(pattern))
    }
    
    /// Check if file is reference material
    fn is_reference_file(&self, path: &str) -> bool {
        let ref_patterns = [".md", ".toml", ".json", ".yaml", ".yml", "README", "docs/"];
        ref_patterns.iter().any(|pattern| path.contains(pattern))
    }
    
    /// Count recent commits for a file
    fn count_recent_commits(&self, repo: &Repository, path: &str) -> Result<usize> {
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        
        let mut count = 0;
        let cutoff = SystemTime::now() - std::time::Duration::from_secs(30 * 24 * 60 * 60); // 30 days
        
        for oid in revwalk.take(100) { // Limit to recent commits
            let commit = repo.find_commit(oid?)?;
            let commit_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(commit.time().seconds() as u64);
            
            if commit_time < cutoff {
                break;
            }
            
            // Check if this commit touched the file
            if let Ok(tree) = commit.tree() {
                if tree.get_path(Path::new(path)).is_ok() {
                    count += 1;
                }
            }
        }
        
        Ok(count)
    }
    
    /// Determine the purpose of a file in the current context
    fn determine_file_purpose(&self, path: &str, file_info: &FileInfo) -> String {
        if file_info.git_status.is_some() {
            return "Currently being modified".to_string();
        }
        
        if path.contains("test") {
            return "Test file".to_string();
        }
        
        if path.contains("config") {
            return "Configuration".to_string();
        }
        
        if path.contains("mod.rs") {
            return "Module definition".to_string();
        }
        
        if path.ends_with(".md") {
            return "Documentation".to_string();
        }
        
        "Implementation file".to_string()
    }
    
    /// Analyze relationship between file and current work
    fn analyze_relationship(&self, path: &str, query: &str) -> String {
        if self.matches_query_context(path, query) {
            return "Directly relevant to current task".to_string();
        }
        
        if self.is_architectural_file(path) {
            return "Core system component".to_string();
        }
        
        if self.is_reference_file(path) {
            return "Reference material".to_string();
        }
        
        "Supporting context".to_string()
    }
    
    /// Predict next actions based on query
    fn predict_next_actions(&self, query: &str) -> Vec<Action> {
        let mut actions = Vec::new();
        
        if query.to_lowercase().contains("test") {
            actions.push(Action {
                action_type: "testing".to_string(),
                description: "Run tests to verify changes".to_string(),
                confidence: 0.8,
            });
        }
        
        if query.to_lowercase().contains("implement") {
            actions.push(Action {
                action_type: "implementation".to_string(),
                description: "Implement new functionality".to_string(),
                confidence: 0.7,
            });
        }
        
        if query.to_lowercase().contains("fix") || query.to_lowercase().contains("bug") {
            actions.push(Action {
                action_type: "debugging".to_string(),
                description: "Debug and fix issues".to_string(),
                confidence: 0.9,
            });
        }
        
        actions
    }

    /// Analyze the impact of changing specific files
    pub async fn analyze_impact(&self, files: &[String]) -> ImpactAnalysis {
        let mut direct_impacts = Vec::new();
        let mut indirect_impacts = Vec::new();
        let mut risk_areas = Vec::new();
        let mut suggested_tests = Vec::new();
        
        for file in files {
            // Find files that directly depend on this file
            let dependents = self.find_direct_dependents(file);
            direct_impacts.extend(dependents);
            
            // Find architectural impacts
            if self.is_architectural_file(file) {
                indirect_impacts.push(format!("Architectural change to {}", file));
                risk_areas.push(format!("Core system modification in {}", file));
            }
            
            // Find test files that might be affected
            let test_files = self.find_related_tests(file);
            suggested_tests.extend(test_files);
            
            // Check if documentation needs updates
            if self.needs_documentation_update(file) {
                indirect_impacts.push(format!("Documentation update needed for {}", file));
            }
        }
        
        ImpactAnalysis {
            direct_impacts,
            indirect_impacts,
            risk_areas,
            suggested_tests,
        }
    }
    
    /// Find files that directly depend on the given file
    fn find_direct_dependents(&self, file: &str) -> Vec<String> {
        let mut dependents = Vec::new();
        
        // Simple heuristic: look for files that might import this module
        let module_name = Path::new(file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        if !module_name.is_empty() {
            for (path, _) in &self.file_cache {
                if path != file && path.contains(module_name) {
                    dependents.push(path.clone());
                }
            }
        }
        
        dependents
    }
    
    /// Find test files related to the given file
    fn find_related_tests(&self, file: &str) -> Vec<String> {
        let mut tests = Vec::new();
        
        let file_name = Path::new(file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        for (path, _) in &self.file_cache {
            if path.contains("test") && path.contains(file_name) {
                tests.push(path.clone());
            }
        }
        
        tests
    }
    
    /// Check if file changes require documentation updates
    fn needs_documentation_update(&self, file: &str) -> bool {
        // Public interfaces, configuration, or core modules likely need doc updates
        file.contains("mod.rs") || 
        file.contains("lib.rs") || 
        file.contains("config") ||
        file.contains("api")
    }

    /// Suggest additional context that might be helpful
    pub async fn suggest_additional_context(&self, current_files: &[String]) -> ContextSuggestions {
        let mut missing_dependencies = Vec::new();
        let mut architectural_context = Vec::new();
        let mut historical_context = Vec::new();
        
        // Find missing dependencies
        for file in current_files {
            let deps = self.find_direct_dependents(file);
            for dep in deps {
                if !current_files.contains(&dep) {
                    missing_dependencies.push(dep);
                }
            }
        }
        
        // Suggest important architectural files not in current context
        for (path, _) in &self.file_cache {
            if self.is_architectural_file(path) && !current_files.contains(path) {
                architectural_context.push(path.clone());
            }
        }
        
        // Suggest configuration and documentation files
        for (path, _) in &self.file_cache {
            if self.is_reference_file(path) && !current_files.contains(path) {
                if path.contains("README") || path.contains("CLAUDE.md") || path.contains(".toml") {
                    historical_context.push(path.clone());
                }
            }
        }
        
        // Limit suggestions to most relevant
        missing_dependencies.truncate(5);
        architectural_context.truncate(5);
        historical_context.truncate(5);
        
        let total_suggestions = missing_dependencies.len() + architectural_context.len() + historical_context.len();
        let confidence = if total_suggestions > 0 { 0.7 } else { 0.0 };
        
        ContextSuggestions {
            missing_dependencies,
            architectural_context,
            historical_context,
            confidence,
        }
    }
}

#[derive(Debug)]
pub struct RelevanceAnalysis {
    pub ranked_files: Vec<FileWithContext>,
    pub confidence: f64,
    pub predicted_actions: Vec<super::Action>,
}