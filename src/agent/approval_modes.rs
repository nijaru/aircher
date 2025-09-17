use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use tracing::{debug, info};

/// Approval modes similar to Claude Code
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ApprovalMode {
    /// Automatically apply all changes without review
    Auto,
    /// Review each change before applying (default)
    Review,
    /// Show diffs only, never apply changes
    DiffOnly,
    /// Smart mode - auto-approve safe ops, review destructive ones
    Smart,
}

impl Default for ApprovalMode {
    fn default() -> Self {
        ApprovalMode::Review
    }
}

/// Type of change being made
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    CreateFile {
        path: PathBuf,
        content: String,
    },
    ModifyFile {
        path: PathBuf,
        old_content: String,
        new_content: String,
    },
    DeleteFile {
        path: PathBuf,
    },
    RunCommand {
        command: String,
        cwd: Option<PathBuf>,
    },
}

/// A pending change that needs approval
#[derive(Debug, Clone)]
pub struct PendingChange {
    pub id: String,
    pub change_type: ChangeType,
    pub tool_name: String,
    pub description: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub is_safe: bool,  // For smart mode
}

impl PendingChange {
    pub fn new(change_type: ChangeType, tool_name: String, description: String) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let is_safe = Self::determine_safety(&change_type);

        Self {
            id,
            change_type,
            tool_name,
            description,
            timestamp: chrono::Utc::now(),
            is_safe,
        }
    }

    /// Determine if a change is safe for auto-approval in Smart mode
    fn determine_safety(change_type: &ChangeType) -> bool {
        match change_type {
            ChangeType::CreateFile { path, .. } => {
                // Safe if creating in project directory, not system files
                !path.starts_with("/etc") &&
                !path.starts_with("/usr") &&
                !path.starts_with("/System")
            }
            ChangeType::ModifyFile { path, .. } => {
                // Safe for common code files
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| matches!(ext, "rs" | "js" | "ts" | "py" | "go" | "java" | "cpp" | "c" | "h"))
                    .unwrap_or(false)
            }
            ChangeType::DeleteFile { .. } => false, // Never auto-approve deletions
            ChangeType::RunCommand { command, .. } => {
                // Safe for read-only commands
                let safe_prefixes = ["ls", "pwd", "echo", "cat", "grep", "find", "git status", "git diff"];
                safe_prefixes.iter().any(|prefix| command.starts_with(prefix))
            }
        }
    }

    /// Generate a diff for this change
    pub fn generate_diff(&self) -> String {
        match &self.change_type {
            ChangeType::CreateFile { path, content } => {
                format!(
                    "--- /dev/null\n+++ {}\n@@ -0,0 +1,{} @@\n{}",
                    path.display(),
                    content.lines().count(),
                    content.lines()
                        .map(|line| format!("+{}", line))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
            ChangeType::ModifyFile { path, old_content, new_content } => {
                // Use similar_text crate or simple line diff
                let old_lines: Vec<&str> = old_content.lines().collect();
                let new_lines: Vec<&str> = new_content.lines().collect();

                let mut diff = format!("--- {}\n+++ {}\n", path.display(), path.display());

                // Simple line-by-line diff (could use better algorithm)
                for (i, (old, new)) in old_lines.iter().zip(new_lines.iter()).enumerate() {
                    if old != new {
                        diff.push_str(&format!("@@ -{},{} +{},{} @@\n", i+1, 1, i+1, 1));
                        diff.push_str(&format!("-{}\n", old));
                        diff.push_str(&format!("+{}\n", new));
                    }
                }

                // Handle added lines
                if new_lines.len() > old_lines.len() {
                    let start = old_lines.len();
                    diff.push_str(&format!("@@ -{},{} +{},{} @@\n",
                        start, 0, start+1, new_lines.len() - start));
                    for line in &new_lines[start..] {
                        diff.push_str(&format!("+{}\n", line));
                    }
                }

                diff
            }
            ChangeType::DeleteFile { path } => {
                format!("--- {}\n+++ /dev/null\n@@ File will be deleted @@", path.display())
            }
            ChangeType::RunCommand { command, cwd } => {
                format!(
                    "$ {}\n# Working directory: {}",
                    command,
                    cwd.as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "current".to_string())
                )
            }
        }
    }
}

/// Applied change history for undo functionality
#[derive(Debug, Clone)]
pub struct AppliedChange {
    pub change: PendingChange,
    pub applied_at: chrono::DateTime<chrono::Utc>,
    pub backup_data: Option<String>, // For undo
}

/// Manages the approval workflow for all changes
pub struct ChangeApprovalManager {
    mode: ApprovalMode,
    pending_changes: VecDeque<PendingChange>,
    applied_changes: Vec<AppliedChange>,
    rejected_changes: Vec<PendingChange>,
    session_approved_patterns: Vec<String>, // For "always approve this"
    max_history: usize,
}

impl ChangeApprovalManager {
    pub fn new(mode: ApprovalMode) -> Self {
        Self {
            mode,
            pending_changes: VecDeque::new(),
            applied_changes: Vec::new(),
            rejected_changes: Vec::new(),
            session_approved_patterns: Vec::new(),
            max_history: 100,
        }
    }

    /// Set the approval mode
    pub fn set_mode(&mut self, mode: ApprovalMode) {
        info!("Approval mode changed to: {:?}", mode);
        self.mode = mode;
    }

    /// Get current approval mode
    pub fn get_mode(&self) -> ApprovalMode {
        self.mode
    }

    /// Add a change for approval
    pub fn queue_change(&mut self, change: PendingChange) -> Result<()> {
        debug!("Queuing change: {} - {}", change.tool_name, change.description);

        // Check if this should be auto-approved
        if self.should_auto_approve(&change) {
            info!("Auto-approving change: {}", change.description);
            self.apply_change_internal(change)?;
        } else {
            self.pending_changes.push_back(change);
        }

        Ok(())
    }

    /// Check if a change should be auto-approved based on mode and patterns
    fn should_auto_approve(&self, change: &PendingChange) -> bool {
        match self.mode {
            ApprovalMode::Auto => true,
            ApprovalMode::DiffOnly => false, // Never auto-approve in diff-only mode
            ApprovalMode::Smart => change.is_safe,
            ApprovalMode::Review => {
                // Check session-approved patterns
                if let ChangeType::RunCommand { command, .. } = &change.change_type {
                    self.session_approved_patterns.iter()
                        .any(|pattern| command.starts_with(pattern))
                } else {
                    false
                }
            }
        }
    }

    /// Get next pending change for review
    pub fn get_next_pending(&mut self) -> Option<PendingChange> {
        self.pending_changes.pop_front()
    }

    /// Get all pending changes
    pub fn get_all_pending(&self) -> &VecDeque<PendingChange> {
        &self.pending_changes
    }

    /// Approve a specific change
    pub fn approve_change(&mut self, change_id: &str) -> Result<()> {
        if let Some(pos) = self.pending_changes.iter().position(|c| c.id == change_id) {
            let change = self.pending_changes.remove(pos).unwrap();
            self.apply_change_internal(change)?;
        }
        Ok(())
    }

    /// Approve all pending changes
    pub fn approve_all(&mut self) -> Result<()> {
        while let Some(change) = self.pending_changes.pop_front() {
            self.apply_change_internal(change)?;
        }
        Ok(())
    }

    /// Reject a specific change
    pub fn reject_change(&mut self, change_id: &str) -> Result<()> {
        if let Some(pos) = self.pending_changes.iter().position(|c| c.id == change_id) {
            let change = self.pending_changes.remove(pos).unwrap();
            info!("Rejected change: {}", change.description);
            self.rejected_changes.push(change);
        }
        Ok(())
    }

    /// Reject all pending changes
    pub fn reject_all(&mut self) -> Result<()> {
        while let Some(change) = self.pending_changes.pop_front() {
            self.rejected_changes.push(change);
        }
        info!("Rejected all {} pending changes", self.rejected_changes.len());
        Ok(())
    }

    /// Apply a change (internal - actually performs the operation)
    fn apply_change_internal(&mut self, change: PendingChange) -> Result<()> {
        // In diff-only mode, we don't actually apply
        if self.mode == ApprovalMode::DiffOnly {
            info!("Diff-only mode: Not applying change {}", change.description);
            return Ok(());
        }

        // Store backup data for undo
        let backup_data = self.create_backup(&change)?;

        // This is where we would actually apply the change
        // For now, we just record it
        info!("Applied change: {}", change.description);

        let applied = AppliedChange {
            change,
            applied_at: chrono::Utc::now(),
            backup_data,
        };

        self.applied_changes.push(applied);

        // Trim history if needed
        if self.applied_changes.len() > self.max_history {
            self.applied_changes.remove(0);
        }

        Ok(())
    }

    /// Create backup data for undo functionality
    fn create_backup(&self, change: &PendingChange) -> Result<Option<String>> {
        match &change.change_type {
            ChangeType::ModifyFile { old_content, .. } => {
                Ok(Some(old_content.clone()))
            }
            ChangeType::DeleteFile { path } => {
                // Would read file content here
                Ok(Some(format!("Backup of {}", path.display())))
            }
            _ => Ok(None),
        }
    }

    /// Undo the last applied change
    pub fn undo_last(&mut self) -> Result<()> {
        if let Some(applied) = self.applied_changes.pop() {
            info!("Undoing change: {}", applied.change.description);
            // Here we would restore from backup_data
            Ok(())
        } else {
            Err(anyhow::anyhow!("No changes to undo"))
        }
    }

    /// Add a pattern for session-level auto-approval
    pub fn add_session_pattern(&mut self, pattern: String) {
        if !self.session_approved_patterns.contains(&pattern) {
            info!("Added session approval pattern: {}", pattern);
            self.session_approved_patterns.push(pattern);
        }
    }

    /// Get statistics about approvals
    pub fn get_stats(&self) -> ApprovalStats {
        ApprovalStats {
            pending: self.pending_changes.len(),
            applied: self.applied_changes.len(),
            rejected: self.rejected_changes.len(),
            mode: self.mode,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApprovalStats {
    pub pending: usize,
    pub applied: usize,
    pub rejected: usize,
    pub mode: ApprovalMode,
}