// Git Snapshot System for Safe Experimentation (Week 7 Day 5)
//
// Implements OpenCode's proven pattern of creating temporary Git commits
// before risky operations, enabling 100% rollback capability.
//
// References:
// - docs/architecture/SYSTEM_DESIGN_2025.md
// - Research: OpenCode uses Git snapshots in production

use anyhow::{Context, Result};
use git2::{
    Oid, Repository, ResetType, Signature, StatusOptions
};
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{debug, info};

use super::events::{AgentEvent, SharedEventBus};

/// Manages Git snapshots for safe experimentation and rollback
pub struct SnapshotManager {
    repo: Mutex<Repository>,
    #[allow(dead_code)]
    workspace_root: PathBuf,
    event_bus: SharedEventBus,
}

impl SnapshotManager {
    /// Create a new snapshot manager for the given workspace
    pub fn new(workspace_root: PathBuf, event_bus: SharedEventBus) -> Result<Self> {
        let repo = Repository::open(&workspace_root)
            .context("Failed to open Git repository. Snapshots require Git.")?;

        info!("Initialized Git snapshot manager for: {:?}", workspace_root);

        Ok(Self {
            repo: Mutex::new(repo),
            workspace_root,
            event_bus,
        })
    }

    /// Create a snapshot of the current state
    ///
    /// Creates a temporary Git commit that can be rolled back to.
    /// The commit is not pushed and doesn't affect the working branch.
    ///
    /// Returns the commit OID for later rollback.
    pub fn create_snapshot(&self, message: &str) -> Result<Oid> {
        debug!("Creating snapshot: {}", message);

        // Lock the repository for this operation
        let repo = self.repo.lock().unwrap();

        // Check if there are any changes to commit
        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);
        status_opts.include_ignored(false);

        let statuses = repo.statuses(Some(&mut status_opts))?;

        if statuses.is_empty() {
            debug!("No changes to snapshot, using current HEAD");
            let head = repo.head()?;
            let commit = head.peel_to_commit()?;
            return Ok(commit.id());
        }

        // Stage all changes (including untracked files)
        let mut index = repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        // Create tree from staged changes
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        // Get parent commit (current HEAD)
        let parent_commit = if let Ok(head) = repo.head() {
            Some(head.peel_to_commit()?)
        } else {
            None // Initial commit case
        };

        // Create signature
        let signature = Signature::now(
            "aircher-snapshot",
            "snapshot@aircher.dev"
        )?;

        // Create commit (detached, doesn't move HEAD)
        let commit_id = if let Some(parent) = &parent_commit {
            repo.commit(
                None, // Don't update any reference (detached)
                &signature,
                &signature,
                &format!("[AIRCHER SNAPSHOT] {}", message),
                &tree,
                &[parent],
            )?
        } else {
            // Initial commit (no parent)
            repo.commit(
                None,
                &signature,
                &signature,
                &format!("[AIRCHER SNAPSHOT] {}", message),
                &tree,
                &[],
            )?
        };

        info!("Created snapshot {}: {}", commit_id, message);

        // Emit event
        self.event_bus.publish(AgentEvent::SnapshotCreated {
            snapshot_id: commit_id.to_string(),
            message: message.to_string(),
            files_changed: statuses.len(),
            timestamp: std::time::SystemTime::now(),
        });

        Ok(commit_id)
    }

    /// Rollback to a previous snapshot
    ///
    /// Performs a hard reset to the specified commit.
    /// All changes since the snapshot will be lost.
    pub fn rollback(&self, snapshot: Oid, reason: &str) -> Result<()> {
        info!("Rolling back to snapshot {}: {}", snapshot, reason);

        // Lock the repository for this operation
        let repo = self.repo.lock().unwrap();

        // Find the commit
        let commit = repo.find_commit(snapshot)
            .context("Snapshot commit not found")?;

        // Hard reset to snapshot
        repo.reset(
            commit.as_object(),
            ResetType::Hard,
            None
        )?;

        info!("Successfully rolled back to snapshot {}", snapshot);

        // Emit event
        self.event_bus.publish(AgentEvent::SnapshotRolledBack {
            snapshot_id: snapshot.to_string(),
            reason: reason.to_string(),
            timestamp: std::time::SystemTime::now(),
        });

        Ok(())
    }

    /// Check if repository has uncommitted changes
    pub fn has_uncommitted_changes(&self) -> Result<bool> {
        let repo = self.repo.lock().unwrap();
        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);
        status_opts.include_ignored(false);

        let statuses = repo.statuses(Some(&mut status_opts))?;
        Ok(!statuses.is_empty())
    }

    /// Get current HEAD commit ID
    pub fn current_head(&self) -> Result<Oid> {
        let repo = self.repo.lock().unwrap();
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.id())
    }

    /// List recent snapshots (commits with [AIRCHER SNAPSHOT] prefix)
    pub fn list_snapshots(&self, limit: usize) -> Result<Vec<SnapshotInfo>> {
        let repo = self.repo.lock().unwrap();
        let mut snapshots = Vec::new();
        let mut revwalk = repo.revwalk()?;

        // Start from HEAD
        if let Ok(head) = repo.head() {
            revwalk.push(head.target().unwrap())?;
        }

        for oid in revwalk.take(limit * 10) { // Search more than limit
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            let message = commit.message().unwrap_or("");

            if message.starts_with("[AIRCHER SNAPSHOT]") {
                let clean_message = message
                    .trim_start_matches("[AIRCHER SNAPSHOT]")
                    .trim();

                snapshots.push(SnapshotInfo {
                    id: oid,
                    message: clean_message.to_string(),
                    timestamp: commit.time(),
                });

                if snapshots.len() >= limit {
                    break;
                }
            }
        }

        Ok(snapshots)
    }

    /// Clean up old snapshots (optional maintenance)
    pub fn cleanup_old_snapshots(&self, keep_count: usize) -> Result<usize> {
        let snapshots = self.list_snapshots(keep_count + 100)?;

        if snapshots.len() <= keep_count {
            return Ok(0);
        }

        let to_delete = &snapshots[keep_count..];
        let mut deleted = 0;

        for snapshot in to_delete {
            // Note: Git commits can't be truly "deleted" without gc
            // This is a placeholder for future implementation
            debug!("Would clean up snapshot: {} ({})", snapshot.id, snapshot.message);
            deleted += 1;
        }

        Ok(deleted)
    }
}

/// Information about a snapshot
#[derive(Debug, Clone)]
pub struct SnapshotInfo {
    pub id: Oid,
    pub message: String,
    pub timestamp: git2::Time,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::events::EventBus;
    use std::fs;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn setup_test_repo() -> Result<(TempDir, PathBuf, Repository)> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().to_path_buf();

        // Initialize git repo
        let repo = Repository::init(&repo_path)?;

        // Configure git
        {
            let mut config = repo.config()?;
            config.set_str("user.name", "Test User")?;
            config.set_str("user.email", "test@example.com")?;
        } // Drop config here so repo is no longer borrowed

        // Create initial commit
        let sig = Signature::now("Test User", "test@example.com")?;
        let tree_id = {
            let mut index = repo.index()?;
            index.write_tree()?
        };
        {
            let tree = repo.find_tree(tree_id)?;
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                "Initial commit",
                &tree,
                &[],
            )?;
        } // Drop tree here so repo is no longer borrowed

        Ok((temp_dir, repo_path, repo))
    }

    #[test]
    fn test_create_snapshot() -> Result<()> {
        let (_temp, repo_path, _repo) = setup_test_repo()?;
        let event_bus = Arc::new(EventBus::new());
        let manager = SnapshotManager::new(repo_path.clone(), event_bus)?;

        // Create a file
        fs::write(repo_path.join("test.txt"), "test content")?;

        // Create snapshot
        let snapshot_id = manager.create_snapshot("Test snapshot")?;
        assert!(snapshot_id.to_string().len() > 0);

        Ok(())
    }

    #[test]
    fn test_rollback() -> Result<()> {
        let (_temp, repo_path, _repo) = setup_test_repo()?;
        let event_bus = Arc::new(EventBus::new());
        let manager = SnapshotManager::new(repo_path.clone(), event_bus)?;

        // Create file and snapshot
        fs::write(repo_path.join("test.txt"), "original")?;
        let snapshot_id = manager.create_snapshot("Before modification")?;

        // Modify file
        fs::write(repo_path.join("test.txt"), "modified")?;

        // Rollback
        manager.rollback(snapshot_id, "Testing rollback")?;

        // Verify file restored
        let content = fs::read_to_string(repo_path.join("test.txt"))?;
        assert_eq!(content, "original");

        Ok(())
    }

    #[test]
    fn test_has_uncommitted_changes() -> Result<()> {
        let (_temp, repo_path, _repo) = setup_test_repo()?;
        let event_bus = Arc::new(EventBus::new());
        let manager = SnapshotManager::new(repo_path.clone(), event_bus)?;

        // No changes initially
        assert!(!manager.has_uncommitted_changes()?);

        // Create file
        fs::write(repo_path.join("test.txt"), "test")?;

        // Should detect changes
        assert!(manager.has_uncommitted_changes()?);

        Ok(())
    }

    #[test]
    fn test_list_snapshots() -> Result<()> {
        let (_temp, repo_path, _repo) = setup_test_repo()?;
        let event_bus = Arc::new(EventBus::new());
        let manager = SnapshotManager::new(repo_path.clone(), event_bus)?;

        // Create multiple snapshots
        fs::write(repo_path.join("test1.txt"), "test1")?;
        manager.create_snapshot("Snapshot 1")?;

        fs::write(repo_path.join("test2.txt"), "test2")?;
        manager.create_snapshot("Snapshot 2")?;

        // List snapshots
        let snapshots = manager.list_snapshots(5)?;
        assert!(snapshots.len() >= 2);
        assert!(snapshots[0].message.contains("Snapshot"));

        Ok(())
    }

    #[test]
    fn test_current_head() -> Result<()> {
        let (_temp, repo_path, repo) = setup_test_repo()?;
        let event_bus = Arc::new(EventBus::new());
        let manager = SnapshotManager::new(repo_path.clone(), event_bus)?;

        let head = manager.current_head()?;
        let repo_head = repo.head()?.peel_to_commit()?.id();

        assert_eq!(head, repo_head);

        Ok(())
    }
}
