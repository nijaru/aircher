// Skill discovery across multiple directories
//
// Scans for SKILL.md files in three locations with precedence:
// 1. Project skills: .aircher/skills/ (highest priority)
// 2. User skills: ~/.aircher/skills/
// 3. System skills: /usr/share/aircher/skills/ (lowest priority)

use crate::agent::skills::metadata::SkillMetadata;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, warn};

/// Discovers skills from configured directories
pub struct SkillDiscovery {
    /// System-wide skills path (e.g., /usr/share/aircher/skills/)
    system_skills_path: PathBuf,

    /// User's global skills path (e.g., ~/.aircher/skills/)
    user_skills_path: PathBuf,

    /// Project-specific skills path (e.g., .aircher/skills/)
    project_skills_path: Option<PathBuf>,
}

impl SkillDiscovery {
    /// Create new SkillDiscovery with default paths
    pub fn new() -> Self {
        let system_path = PathBuf::from("/usr/share/aircher/skills");
        let user_path = Self::default_user_path();
        let project_path = Self::default_project_path();

        Self {
            system_skills_path: system_path,
            user_skills_path: user_path,
            project_skills_path: project_path,
        }
    }

    /// Create SkillDiscovery with custom paths
    pub fn with_paths(
        system_path: PathBuf,
        user_path: PathBuf,
        project_path: Option<PathBuf>,
    ) -> Self {
        Self {
            system_skills_path: system_path,
            user_skills_path: user_path,
            project_skills_path: project_path,
        }
    }

    /// Get default user skills path (~/.aircher/skills/)
    fn default_user_path() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            home.join(".aircher").join("skills")
        } else {
            PathBuf::from(".aircher/skills")
        }
    }

    /// Get default project skills path (.aircher/skills/ in current directory)
    fn default_project_path() -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok()?;
        let project_path = cwd.join(".aircher").join("skills");

        if project_path.exists() {
            Some(project_path)
        } else {
            None
        }
    }

    /// Discover all skills from configured directories
    ///
    /// Skills are discovered in order of precedence (project > user > system).
    /// If multiple skills have the same name, earlier paths take precedence.
    pub async fn discover_all(&self) -> Result<Vec<SkillMetadata>> {
        let mut all_skills = Vec::new();
        let mut seen_names = HashSet::new();

        // Scan in order of precedence (project > user > system)
        // Project skills (highest priority)
        if let Some(project_path) = &self.project_skills_path {
            debug!("Scanning project skills: {}", project_path.display());
            let project_skills = self.scan_directory(project_path).await?;
            for skill in project_skills {
                if seen_names.insert(skill.name.clone()) {
                    all_skills.push(skill);
                } else {
                    debug!("Skipping duplicate skill (project): {}", skill.name);
                }
            }
        }

        // User skills
        debug!("Scanning user skills: {}", self.user_skills_path.display());
        let user_skills = self.scan_directory(&self.user_skills_path).await?;
        for skill in user_skills {
            if seen_names.insert(skill.name.clone()) {
                all_skills.push(skill);
            } else {
                debug!("Skipping duplicate skill (user): {}", skill.name);
            }
        }

        // System skills (lowest priority)
        debug!("Scanning system skills: {}", self.system_skills_path.display());
        let system_skills = self.scan_directory(&self.system_skills_path).await?;
        for skill in system_skills {
            if seen_names.insert(skill.name.clone()) {
                all_skills.push(skill);
            } else {
                debug!("Skipping duplicate skill (system): {}", skill.name);
            }
        }

        debug!("Discovered {} unique skills", all_skills.len());

        Ok(all_skills)
    }

    /// Scan a directory for SKILL.md files
    ///
    /// Each skill is in its own subdirectory with a SKILL.md file:
    /// ```
    /// skills/
    /// ├── search_documentation/
    /// │   └── SKILL.md
    /// └── deploy_to_staging/
    ///     └── SKILL.md
    /// ```
    async fn scan_directory(&self, path: &PathBuf) -> Result<Vec<SkillMetadata>> {
        // Check if directory exists
        if !path.exists() {
            debug!("Skills directory does not exist: {}", path.display());
            return Ok(Vec::new());
        }

        let mut skills = Vec::new();

        // Read directory entries
        let mut entries = fs::read_dir(path)
            .await
            .with_context(|| format!("Failed to read skills directory: {}", path.display()))?;

        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();

            // Skip non-directories
            if !entry_path.is_dir() {
                continue;
            }

            // Look for SKILL.md file
            let skill_file = entry_path.join("SKILL.md");

            if !skill_file.exists() {
                debug!("Skipping directory without SKILL.md: {}", entry_path.display());
                continue;
            }

            // Parse skill metadata
            match SkillMetadata::from_file(skill_file.clone()).await {
                Ok(metadata) => {
                    debug!("Discovered skill: {} ({})", metadata.name, skill_file.display());
                    skills.push(metadata);
                }
                Err(e) => {
                    warn!(
                        "Failed to parse skill at {}: {}",
                        skill_file.display(),
                        e
                    );
                }
            }
        }

        Ok(skills)
    }

    /// Discover skills matching specific tags
    pub async fn discover_by_tags(&self, tags: &[String]) -> Result<Vec<SkillMetadata>> {
        let all_skills = self.discover_all().await?;

        let filtered_skills: Vec<_> = all_skills
            .into_iter()
            .filter(|skill| {
                // Skill matches if it has ANY of the requested tags
                tags.iter().any(|tag| skill.tags.contains(tag))
            })
            .collect();

        Ok(filtered_skills)
    }

    /// Discover skills with specific capabilities
    pub async fn discover_by_capability(&self, capability: &str) -> Result<Vec<SkillMetadata>> {
        let all_skills = self.discover_all().await?;

        let filtered_skills: Vec<_> = all_skills
            .into_iter()
            .filter(|skill| skill.capabilities.contains(&capability.to_string()))
            .collect();

        Ok(filtered_skills)
    }

    /// Get the effective path for a skill name (respecting precedence)
    pub async fn get_skill_path(&self, name: &str) -> Option<PathBuf> {
        // Check project path first
        if let Some(project_path) = &self.project_skills_path {
            let skill_dir = project_path.join(name);
            let skill_file = skill_dir.join("SKILL.md");
            if skill_file.exists() {
                return Some(skill_file);
            }
        }

        // Check user path
        let user_skill_dir = self.user_skills_path.join(name);
        let user_skill_file = user_skill_dir.join("SKILL.md");
        if user_skill_file.exists() {
            return Some(user_skill_file);
        }

        // Check system path
        let system_skill_dir = self.system_skills_path.join(name);
        let system_skill_file = system_skill_dir.join("SKILL.md");
        if system_skill_file.exists() {
            return Some(system_skill_file);
        }

        None
    }
}

impl Default for SkillDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_skill(dir: &PathBuf, name: &str, description: &str) {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).await.unwrap();

        let content = format!(
            r#"---
name: {}
description: {}
version: 1.0.0
---
# {}
Documentation for {}.
"#,
            name, description, name, name
        );

        fs::write(skill_dir.join("SKILL.md"), content)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_discover_all_empty() {
        let temp_dir = TempDir::new().unwrap();
        let discovery = SkillDiscovery::with_paths(
            temp_dir.path().to_path_buf(),
            temp_dir.path().to_path_buf(),
            None,
        );

        let skills = discovery.discover_all().await.unwrap();
        assert_eq!(skills.len(), 0);
    }

    #[tokio::test]
    async fn test_discover_all_with_skills() {
        let temp_dir = TempDir::new().unwrap();

        // Create test skills
        create_test_skill(&temp_dir.path().to_path_buf(), "skill-one", "First skill").await;
        create_test_skill(&temp_dir.path().to_path_buf(), "skill-two", "Second skill").await;

        let discovery = SkillDiscovery::with_paths(
            temp_dir.path().to_path_buf(),
            temp_dir.path().to_path_buf(),
            None,
        );

        let skills = discovery.discover_all().await.unwrap();
        assert_eq!(skills.len(), 2);

        let names: Vec<_> = skills.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"skill-one"));
        assert!(names.contains(&"skill-two"));
    }

    #[tokio::test]
    async fn test_precedence_project_over_user() {
        let temp_dir = TempDir::new().unwrap();
        let user_dir = temp_dir.path().join("user");
        let project_dir = temp_dir.path().join("project");

        fs::create_dir_all(&user_dir).await.unwrap();
        fs::create_dir_all(&project_dir).await.unwrap();

        // Create same skill in both directories with different descriptions
        create_test_skill(&user_dir, "test-skill", "User version").await;
        create_test_skill(&project_dir, "test-skill", "Project version").await;

        let discovery = SkillDiscovery::with_paths(
            temp_dir.path().to_path_buf(),
            user_dir.clone(),
            Some(project_dir.clone()),
        );

        let skills = discovery.discover_all().await.unwrap();

        // Should only find one skill (project takes precedence)
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-skill");
        assert_eq!(skills[0].description, "Project version");
    }

    #[tokio::test]
    async fn test_discover_by_tags() {
        let temp_dir = TempDir::new().unwrap();

        // Create skill with tags
        let skill_dir = temp_dir.path().join("tagged-skill");
        fs::create_dir_all(&skill_dir).await.unwrap();

        let content = r#"---
name: tagged-skill
description: A skill with tags
version: 1.0.0
tags:
  - testing
  - deployment
---
# Tagged Skill
"#;
        fs::write(skill_dir.join("SKILL.md"), content)
            .await
            .unwrap();

        let discovery = SkillDiscovery::with_paths(
            temp_dir.path().to_path_buf(),
            temp_dir.path().to_path_buf(),
            None,
        );

        // Discover by tags
        let skills = discovery
            .discover_by_tags(&["testing".to_string()])
            .await
            .unwrap();
        assert_eq!(skills.len(), 1);

        let skills = discovery
            .discover_by_tags(&["deployment".to_string()])
            .await
            .unwrap();
        assert_eq!(skills.len(), 1);

        let skills = discovery
            .discover_by_tags(&["nonexistent".to_string()])
            .await
            .unwrap();
        assert_eq!(skills.len(), 0);
    }

    #[tokio::test]
    async fn test_discover_by_capability() {
        let temp_dir = TempDir::new().unwrap();

        // Create skill with capabilities
        let skill_dir = temp_dir.path().join("capable-skill");
        fs::create_dir_all(&skill_dir).await.unwrap();

        let content = r#"---
name: capable-skill
description: A skill with capabilities
version: 1.0.0
capabilities:
  - read_files
  - run_commands
---
# Capable Skill
"#;
        fs::write(skill_dir.join("SKILL.md"), content)
            .await
            .unwrap();

        let discovery = SkillDiscovery::with_paths(
            temp_dir.path().to_path_buf(),
            temp_dir.path().to_path_buf(),
            None,
        );

        // Discover by capability
        let skills = discovery
            .discover_by_capability("read_files")
            .await
            .unwrap();
        assert_eq!(skills.len(), 1);

        let skills = discovery
            .discover_by_capability("run_commands")
            .await
            .unwrap();
        assert_eq!(skills.len(), 1);

        let skills = discovery
            .discover_by_capability("nonexistent")
            .await
            .unwrap();
        assert_eq!(skills.len(), 0);
    }

    #[tokio::test]
    async fn test_get_skill_path() {
        let temp_dir = TempDir::new().unwrap();
        let user_dir = temp_dir.path().join("user");
        let project_dir = temp_dir.path().join("project");

        fs::create_dir_all(&user_dir).await.unwrap();
        fs::create_dir_all(&project_dir).await.unwrap();

        // Create skills in different directories
        create_test_skill(&user_dir, "user-skill", "User skill").await;
        create_test_skill(&project_dir, "project-skill", "Project skill").await;

        let discovery = SkillDiscovery::with_paths(
            temp_dir.path().to_path_buf(),
            user_dir.clone(),
            Some(project_dir.clone()),
        );

        // Get skill paths
        let user_path = discovery.get_skill_path("user-skill").await;
        assert!(user_path.is_some());
        assert!(user_path.unwrap().ends_with("user/user-skill/SKILL.md"));

        let project_path = discovery.get_skill_path("project-skill").await;
        assert!(project_path.is_some());
        assert!(project_path.unwrap().ends_with("project/project-skill/SKILL.md"));

        let nonexistent_path = discovery.get_skill_path("nonexistent").await;
        assert!(nonexistent_path.is_none());
    }
}
