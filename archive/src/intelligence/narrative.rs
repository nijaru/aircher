use crate::config::ConfigManager;
use anyhow::Result;
use super::{ArchitecturalDecision, ProjectMomentum};
use git2::Repository;
use chrono::{DateTime, Utc};
use serde_json;
use std::sync::{Arc, Mutex};

/// Development Narrative Tracker - Maintains the story of the codebase
pub struct DevelopmentNarrativeTracker {
    _config: ConfigManager,
    project_root: std::path::PathBuf,
    git_repo: Arc<Mutex<Option<Repository>>>,
    _cached_narrative: Option<DevelopmentNarrative>,
}

impl DevelopmentNarrativeTracker {
    pub async fn new(config: &ConfigManager) -> Result<Self> {
        let project_root = std::env::current_dir()?;
        let git_repo = Arc::new(Mutex::new(Repository::open(&project_root).ok()));

        Ok(Self {
            _config: config.clone(),
            project_root,
            git_repo,
            _cached_narrative: None,
        })
    }

    /// Get the current development narrative
    pub async fn get_current_narrative(&self) -> DevelopmentNarrative {
        // Analyze project state to build narrative
        let current_epic = self.determine_current_epic().await;
        let recent_focus = self.analyze_recent_focus().await;
        let recent_decisions = self.extract_recent_decisions().await;

        DevelopmentNarrative {
            current_epic,
            recent_focus,
            recent_decisions,
        }
    }

    /// Determine the current epic/major feature being worked on
    async fn determine_current_epic(&self) -> String {
        // Check tasks.json for current focus
        let tasks_path = self.project_root.join("docs/tasks/tasks.json");
        if tasks_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&tasks_path) {
                if let Ok(tasks) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(current_focus) = tasks["priorities"]["current_focus"].as_str() {
                        return current_focus.to_string();
                    }
                }
            }
        }

        // Fallback to recent commit analysis
        if let Ok(repo_guard) = self.git_repo.lock() {
            if let Some(repo) = repo_guard.as_ref() {
                if let Ok(head) = repo.head() {
                    if let Ok(commit) = head.peel_to_commit() {
                        let message = commit.message().unwrap_or("");
                        if !message.is_empty() {
                            return format!("Recent work: {}", message.lines().next().unwrap_or(""));
                        }
                    }
                }
            }
        }

        "Development in progress".to_string()
    }

    /// Analyze recent focus areas
    async fn analyze_recent_focus(&self) -> String {
        let mut recent_files = std::collections::HashMap::new();

        if let Ok(repo_guard) = self.git_repo.lock() {
            if let Some(repo) = repo_guard.as_ref() {
                // Analyze recent commits to understand focus areas
                if let Ok(mut revwalk) = repo.revwalk() {
                    let _ = revwalk.push_head();

                    for oid in revwalk.take(10) {
                        if let Ok(commit) = repo.find_commit(oid.unwrap()) {
                            if let Ok(tree) = commit.tree() {
                                tree.walk(git2::TreeWalkMode::PreOrder, |root, entry| {
                                    if let Some(name) = entry.name() {
                                        let full_path = if root.is_empty() {
                                            name.to_string()
                                        } else {
                                            format!("{}/{}", root, name)
                                        };

                                        if full_path.starts_with("src/") {
                                            let count = recent_files.entry(full_path).or_insert(0);
                                            *count += 1;
                                        }
                                    }
                                    git2::TreeWalkResult::Ok
                                }).ok();
                            }
                        }
                    }
                }
            }
        }

        // Find the most frequently changed area
        if let Some((path, _)) = recent_files.iter().max_by_key(|(_, count)| *count) {
            if path.contains("ui/") {
                return "User Interface Development".to_string();
            } else if path.contains("providers/") {
                return "API Provider Implementation".to_string();
            } else if path.contains("intelligence/") {
                return "Intelligence Engine Development".to_string();
            } else if path.contains("cli/") {
                return "Command Line Interface".to_string();
            }
        }

        "Active development".to_string()
    }

    /// Extract recent architectural decisions from commits and documentation
    async fn extract_recent_decisions(&self) -> Vec<ArchitecturalDecision> {
        let mut decisions = Vec::new();

        if let Ok(repo_guard) = self.git_repo.lock() {
            if let Some(repo) = repo_guard.as_ref() {
                if let Ok(mut revwalk) = repo.revwalk() {
                    let _ = revwalk.push_head();

                    for oid in revwalk.take(20) {
                        if let Ok(commit) = repo.find_commit(oid.unwrap()) {
                            let message = commit.message().unwrap_or("");

                            // Look for architectural decision patterns in commit messages
                            if self.is_architectural_commit(message) {
                                decisions.push(ArchitecturalDecision {
                                    decision: message.lines().next().unwrap_or("").to_string(),
                                    rationale: self.extract_rationale(message),
                                    affected_files: self.get_commit_files(&commit),
                                    implications: self.infer_implications(message),
                                    timestamp: DateTime::from_timestamp(commit.time().seconds(), 0)
                                        .unwrap_or_else(|| Utc::now()),
                                });
                            }
                        }
                    }
                }
            }
        }

        decisions.truncate(5); // Keep only the 5 most recent decisions
        decisions
    }

    /// Check if commit message indicates an architectural decision
    fn is_architectural_commit(&self, message: &str) -> bool {
        let arch_keywords = [
            "implement", "architecture", "design", "refactor", "restructure",
            "add", "create", "engine", "system", "framework", "provider",
            "intelligence", "context", "narrative", "memory"
        ];

        let msg_lower = message.to_lowercase();
        arch_keywords.iter().any(|keyword| msg_lower.contains(keyword))
    }

    /// Extract rationale from commit message
    fn extract_rationale(&self, message: &str) -> String {
        let lines: Vec<&str> = message.lines().collect();
        if lines.len() > 2 {
            // Look for extended commit message as rationale
            lines[2..].join(" ").trim().to_string()
        } else {
            "Architectural improvement".to_string()
        }
    }

    /// Get files affected by a commit
    fn get_commit_files(&self, commit: &git2::Commit) -> Vec<String> {
        let mut files = Vec::new();

        if let Ok(tree) = commit.tree() {
            let _ = tree.walk(git2::TreeWalkMode::PreOrder, |root, entry| {
                if let Some(name) = entry.name() {
                    let full_path = if root.is_empty() {
                        name.to_string()
                    } else {
                        format!("{}/{}", root, name)
                    };
                    files.push(full_path);
                }
                git2::TreeWalkResult::Ok
            });
        }

        files.truncate(10); // Limit to avoid huge lists
        files
    }

    /// Infer implications from commit message
    fn infer_implications(&self, message: &str) -> Vec<String> {
        let mut implications = Vec::new();

        if message.to_lowercase().contains("test") {
            implications.push("Testing strategy updated".to_string());
        }

        if message.to_lowercase().contains("provider") {
            implications.push("API integration patterns established".to_string());
        }

        if message.to_lowercase().contains("ui") || message.to_lowercase().contains("tui") {
            implications.push("User interface patterns established".to_string());
        }

        if message.to_lowercase().contains("intelligence") {
            implications.push("Context analysis capabilities added".to_string());
        }

        if implications.is_empty() {
            implications.push("System architecture evolved".to_string());
        }

        implications
    }

    /// Get project momentum and direction indicators
    pub async fn get_momentum(&self) -> ProjectMomentum {
        let recent_focus = self.analyze_recent_focus().await;
        let velocity_indicators = self.analyze_velocity().await;
        let architectural_direction = self.determine_architectural_direction().await;
        let next_priorities = self.predict_next_priorities().await;
        let knowledge_gaps = self.identify_knowledge_gaps().await;

        ProjectMomentum {
            recent_focus,
            velocity_indicators,
            architectural_direction,
            next_priorities,
            knowledge_gaps,
        }
    }

    /// Analyze development velocity indicators
    async fn analyze_velocity(&self) -> Vec<String> {
        let mut indicators = Vec::new();
        let mut recent_commits = 0;
        let mut file_changes = std::collections::HashMap::new();

        if let Ok(repo_guard) = self.git_repo.lock() {
            if let Some(repo) = repo_guard.as_ref() {
                // Count commits in the last week
                let week_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(7 * 24 * 60 * 60);

                if let Ok(mut revwalk) = repo.revwalk() {
                    let _ = revwalk.push_head();

                    for oid in revwalk.take(50) {
                        if let Ok(commit) = repo.find_commit(oid.unwrap()) {
                            let commit_time = std::time::SystemTime::UNIX_EPOCH +
                                std::time::Duration::from_secs(commit.time().seconds() as u64);

                            if commit_time > week_ago {
                                recent_commits += 1;
                            } else {
                                break;
                            }
                        }
                    }
                }

                // Analyze file change patterns
                if let Ok(mut revwalk) = repo.revwalk() {
                    let _ = revwalk.push_head();

                    for oid in revwalk.take(10) {
                        if let Ok(commit) = repo.find_commit(oid.unwrap()) {
                            if let Ok(tree) = commit.tree() {
                                let _ = tree.walk(git2::TreeWalkMode::PreOrder, |root, entry| {
                                    if let Some(name) = entry.name() {
                                        let full_path = if root.is_empty() {
                                            name.to_string()
                                        } else {
                                            format!("{}/{}", root, name)
                                        };

                                        if full_path.starts_with("src/") {
                                            let area = full_path.split('/').nth(1).unwrap_or("");
                                            let count = file_changes.entry(area.to_string()).or_insert(0);
                                            *count += 1;
                                        }
                                    }
                                    git2::TreeWalkResult::Ok
                                });
                            }
                        }
                    }
                }
            }
        }

        // Analyze commit velocity
        if recent_commits > 10 {
            indicators.push("High development velocity".to_string());
        } else if recent_commits > 5 {
            indicators.push("Moderate development velocity".to_string());
        } else {
            indicators.push("Steady development pace".to_string());
        }

        // Report on active areas
        for (area, count) in file_changes.iter() {
            if *count > 3 {
                indicators.push(format!("Active development in {}/", area));
            }
        }

        if indicators.is_empty() {
            indicators.push("Development in progress".to_string());
        }

        indicators
    }

    /// Determine architectural direction
    async fn determine_architectural_direction(&self) -> String {
        // Look for architectural patterns in recent commits
        if let Ok(repo_guard) = self.git_repo.lock() {
            if let Some(repo) = repo_guard.as_ref() {
                if let Ok(mut revwalk) = repo.revwalk() {
                    let _ = revwalk.push_head();

                    for oid in revwalk.take(20) {
                        if let Ok(commit) = repo.find_commit(oid.unwrap()) {
                            let message = commit.message().unwrap_or("");

                            if message.to_lowercase().contains("pure rust") {
                                return "Pure Rust single binary architecture".to_string();
                            }

                            if message.to_lowercase().contains("intelligence") {
                                return "AI-first intelligent development assistant".to_string();
                            }

                            if message.to_lowercase().contains("provider") {
                                return "Multi-provider API architecture".to_string();
                            }
                        }
                    }
                }
            }
        }

        "Modular Rust architecture with clean separation of concerns".to_string()
    }

    /// Predict next priorities based on current state
    async fn predict_next_priorities(&self) -> Vec<String> {
        let mut priorities = Vec::new();

        // Check tasks.json for next sequence
        let tasks_path = self.project_root.join("docs/tasks/tasks.json");
        if tasks_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&tasks_path) {
                if let Ok(tasks) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(next_seq) = tasks["priorities"]["next_sequence"].as_array() {
                        for task_id in next_seq.iter().take(3) {
                            if let Some(task_str) = task_id.as_str() {
                                if let Some(task) = tasks["tasks"][task_str].as_object() {
                                    if let Some(title) = task["title"].as_str() {
                                        priorities.push(title.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if priorities.is_empty() {
            priorities.push("Intelligence Engine implementation".to_string());
            priorities.push("Integration testing".to_string());
            priorities.push("Advanced features".to_string());
        }

        priorities
    }

    /// Identify knowledge gaps and areas needing attention
    async fn identify_knowledge_gaps(&self) -> Vec<String> {
        let mut gaps = Vec::new();

        // Check for TODOs in intelligence module
        let intelligence_path = self.project_root.join("src/intelligence/");
        if intelligence_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&intelligence_path) {
                for entry in entries.flatten() {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if content.contains("TODO") {
                            gaps.push("Intelligence Engine implementation incomplete".to_string());
                            break;
                        }
                    }
                }
            }
        }

        // Check test coverage
        let tests_path = self.project_root.join("tests/");
        if !tests_path.exists() {
            gaps.push("Test coverage needs improvement".to_string());
        }

        // Check documentation
        let docs_path = self.project_root.join("docs/");
        if !docs_path.exists() {
            gaps.push("Documentation structure needs development".to_string());
        }

        if gaps.is_empty() {
            gaps.push("System architecture well-defined".to_string());
        }

        gaps
    }

    /// Record a new architectural decision
    pub async fn record_decision(&self, decision: ArchitecturalDecision) {
        // For now, we'll rely on git commits to track decisions
        // In a full implementation, we'd store these in a database
        // or structured decision log file

        tracing::info!(
            "Architectural decision recorded: {} - {}",
            decision.decision,
            decision.rationale
        );
    }
}

#[derive(Debug)]
pub struct DevelopmentNarrative {
    pub current_epic: String,
    pub recent_focus: String,
    pub recent_decisions: Vec<ArchitecturalDecision>,
}
