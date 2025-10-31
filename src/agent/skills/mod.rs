// Skills system for user-extensible agent capabilities
//
// Based on SOTA analysis of Claude Agent Skills and TUI agent features.
// Enables users to define custom agent capabilities via SKILL.md files
// without modifying Rust code.

pub mod metadata;
pub mod discovery;
pub mod tool;
pub mod executor;

pub use metadata::{SkillMetadata, ParameterSchema, ParameterType};
pub use discovery::SkillDiscovery;
pub use tool::SkillTool;
pub use executor::{SkillExecutor, SkillContext};

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manager for the Skills system
///
/// Handles skill discovery, caching, and lifecycle management.
pub struct SkillManager {
    /// Skill discovery engine
    discovery: SkillDiscovery,

    /// Cached discovered skills
    cached_skills: Arc<RwLock<Vec<SkillMetadata>>>,

    /// Whether skills have been loaded
    loaded: Arc<RwLock<bool>>,
}

impl SkillManager {
    /// Create a new SkillManager with default paths
    pub fn new() -> Self {
        Self {
            discovery: SkillDiscovery::new(),
            cached_skills: Arc::new(RwLock::new(Vec::new())),
            loaded: Arc::new(RwLock::new(false)),
        }
    }

    /// Create a SkillManager with custom paths
    pub fn with_paths(
        system_path: PathBuf,
        user_path: PathBuf,
        project_path: Option<PathBuf>,
    ) -> Self {
        Self {
            discovery: SkillDiscovery::with_paths(system_path, user_path, project_path),
            cached_skills: Arc::new(RwLock::new(Vec::new())),
            loaded: Arc::new(RwLock::new(false)),
        }
    }

    /// Discover and load all available skills
    ///
    /// This scans all configured directories and caches the results.
    /// Subsequent calls return cached skills unless force_reload is true.
    pub async fn load_skills(&self, force_reload: bool) -> Result<Vec<SkillMetadata>> {
        let mut loaded = self.loaded.write().await;

        if *loaded && !force_reload {
            let cached = self.cached_skills.read().await;
            return Ok(cached.clone());
        }

        // Discover all skills
        let skills = self.discovery.discover_all().await?;

        // Cache results
        let mut cached = self.cached_skills.write().await;
        *cached = skills.clone();
        *loaded = true;

        Ok(skills)
    }

    /// List all discovered skills
    pub async fn list_skills(&self) -> Result<Vec<SkillMetadata>> {
        self.load_skills(false).await
    }

    /// Get a specific skill by name
    pub async fn get_skill(&self, name: &str) -> Result<Option<SkillMetadata>> {
        let skills = self.load_skills(false).await?;
        Ok(skills.into_iter().find(|s| s.name == name))
    }

    /// Reload skills from disk
    pub async fn reload(&self) -> Result<Vec<SkillMetadata>> {
        self.load_skills(true).await
    }
}

impl Default for SkillManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_skill_manager_empty() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SkillManager::with_paths(
            temp_dir.path().to_path_buf(),
            temp_dir.path().to_path_buf(),
            None,
        );

        let skills = manager.list_skills().await.unwrap();
        assert_eq!(skills.len(), 0);
    }

    #[tokio::test]
    async fn test_skill_manager_caching() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("test_skill");
        fs::create_dir(&skill_dir).await.unwrap();

        let skill_content = r#"---
name: test_skill
description: A test skill
version: 1.0.0
---
# Test Skill
Test documentation.
"#;
        fs::write(skill_dir.join("SKILL.md"), skill_content).await.unwrap();

        let manager = SkillManager::with_paths(
            temp_dir.path().to_path_buf(),
            temp_dir.path().to_path_buf(),
            None,
        );

        // First load
        let skills1 = manager.list_skills().await.unwrap();
        assert_eq!(skills1.len(), 1);

        // Second load should use cache
        let skills2 = manager.list_skills().await.unwrap();
        assert_eq!(skills2.len(), 1);

        // Get specific skill
        let skill = manager.get_skill("test_skill").await.unwrap();
        assert!(skill.is_some());
        assert_eq!(skill.unwrap().name, "test_skill");
    }
}
