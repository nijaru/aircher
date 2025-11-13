use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;
use tracing::{debug, info};

/// Project detection and management
#[derive(Clone)]
pub struct ProjectManager {
    current_dir: PathBuf,
    project_root: Option<PathBuf>,
}

impl ProjectManager {
    /// Create a new project manager for the current directory
    pub fn new() -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        let project_root = Self::find_project_root(&current_dir)?;

        Ok(Self {
            current_dir,
            project_root,
        })
    }

    /// Find the project root by looking for .aircher/ directory
    /// Similar to git's directory traversal
    fn find_project_root(start_dir: &Path) -> Result<Option<PathBuf>> {
        let mut current = start_dir;

        loop {
            let aircher_dir = current.join(".aircher");
            if aircher_dir.exists() && aircher_dir.is_dir() {
                debug!("Found .aircher directory at: {}", current.display());
                return Ok(Some(current.to_path_buf()));
            }

            // Move up one directory
            match current.parent() {
                Some(parent) => current = parent,
                None => break,
            }
        }

        debug!("No .aircher directory found in hierarchy");
        Ok(None)
    }

    /// Get the project root directory
    pub fn get_project_root(&self) -> Option<&Path> {
        self.project_root.as_deref()
    }

    /// Get the current working directory
    pub fn get_current_dir(&self) -> &Path {
        &self.current_dir
    }

    /// Initialize a new project in the current directory
    pub fn initialize_project(&mut self) -> Result<PathBuf> {
        let project_root = if let Some(root) = &self.project_root {
            root.clone()
        } else {
            self.current_dir.clone()
        };

        let aircher_dir = project_root.join(".aircher");

        // Create .aircher directory if it doesn't exist
        if !aircher_dir.exists() {
            fs::create_dir_all(&aircher_dir)?;
            info!("Created .aircher directory at: {}", aircher_dir.display());
        }

        // Create essential files
        self.create_essential_files(&aircher_dir)?;

        // Update project root
        self.project_root = Some(project_root.clone());

        Ok(project_root)
    }

    /// Create essential files for a new project
    fn create_essential_files(&self, aircher_dir: &Path) -> Result<()> {
        // Create AGENT.md if it doesn't exist
        let agent_file = aircher_dir.join("AGENT.md");
        if !agent_file.exists() {
            let agent_content = self.default_agent_content();
            fs::write(&agent_file, agent_content)?;
            info!("Created AGENT.md file");
        }

        // Create sessions database directory
        let sessions_dir = aircher_dir.join("sessions");
        if !sessions_dir.exists() {
            fs::create_dir_all(&sessions_dir)?;
            info!("Created sessions directory");
        }

        // Create intelligence cache directory
        let intelligence_dir = aircher_dir.join("intelligence");
        if !intelligence_dir.exists() {
            fs::create_dir_all(&intelligence_dir)?;
            info!("Created intelligence directory");
        }

        // Create .gitignore for .aircher directory
        let gitignore_file = aircher_dir.join(".gitignore");
        if !gitignore_file.exists() {
            let gitignore_content = r#"# Aircher cache and temporary files
sessions/
intelligence/
*.db
*.log
.tmp/
"#;
            fs::write(&gitignore_file, gitignore_content)?;
            info!("Created .gitignore file");
        }

        Ok(())
    }

    /// Generate default AGENT.md content
    fn default_agent_content(&self) -> String {
        let project_name = self.current_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project");

        format!(r#"# {} Agent Configuration

## Project Context

This is a {} project. The AI assistant should understand the following:

### Project Structure
- Brief description of the project's purpose
- Key directories and their purposes
- Important files and their roles

### Development Guidelines
- Coding standards and conventions
- Testing approach and requirements
- Build and deployment processes

### AI Assistant Instructions
- Preferred communication style
- Specific tasks the AI should help with
- Files to pay special attention to
- Common patterns and practices in this codebase

## Context for AI

Replace this section with project-specific information that will help the AI understand your codebase better.
"#, project_name, project_name)
    }

    /// Get the path to the sessions database
    pub fn get_sessions_db_path(&self) -> Result<PathBuf> {
        let root = self.get_project_root_or_current();
        Ok(root.join(".aircher").join("sessions").join("sessions.db"))
    }

    /// Get the path to the intelligence cache
    pub fn get_intelligence_cache_path(&self) -> Result<PathBuf> {
        let root = self.get_project_root_or_current();
        Ok(root.join(".aircher").join("intelligence"))
    }

    /// Get the path to the agent configuration file
    pub fn get_agent_config_path(&self) -> Result<PathBuf> {
        let root = self.get_project_root_or_current();
        Ok(root.join(".aircher").join("AGENT.md"))
    }

    /// Get project root or current directory as fallback
    fn get_project_root_or_current(&self) -> &Path {
        self.project_root.as_deref().unwrap_or(&self.current_dir)
    }

    /// Check if we're in a valid project
    pub fn is_project_initialized(&self) -> bool {
        self.project_root.is_some()
    }

    /// Get project metadata
    pub fn get_project_info(&self) -> ProjectInfo {
        let root = self.get_project_root_or_current();
        let name = root.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        ProjectInfo {
            name,
            root_path: root.to_path_buf(),
            is_initialized: self.is_project_initialized(),
            has_git: root.join(".git").exists(),
            agent_config_exists: self.get_agent_config_path()
                .map(|p| p.exists())
                .unwrap_or(false),
        }
    }
}

/// Project information and metadata
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub name: String,
    pub root_path: PathBuf,
    pub is_initialized: bool,
    pub has_git: bool,
    pub agent_config_exists: bool,
}
