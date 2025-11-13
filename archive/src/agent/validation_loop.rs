//! Validation Loop Coordinator for AutoGen-style multi-agent verification
//!
//! This module implements a validation loop pattern to improve location identification
//! in code modification tasks. It uses two specialized agents:
//! - Explorer: Finds candidate locations and verifies proposals
//! - Builder: Generates patches for verified locations
//!
//! The loop continues until a verified location is found or max attempts are reached.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Maximum number of location candidates to try before giving up
const DEFAULT_MAX_ATTEMPTS: usize = 3;

/// A candidate location where a bug might exist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationCandidate {
    /// Path to the file containing the bug
    pub file_path: PathBuf,

    /// Optional line number if identifiable
    pub line_number: Option<usize>,

    /// Confidence score from 0.0 to 1.0
    pub confidence: f32,

    /// Reasoning for why this location was chosen
    pub reasoning: String,
}

/// A proposed patch for a specific location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchProposal {
    /// The location this patch targets
    pub location: LocationCandidate,

    /// Unified diff format patch
    pub patch: String,

    /// Explanation of what the patch does and why
    pub reasoning: String,
}

/// Result of verifying a patch location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether the location is correct
    pub is_correct: bool,

    /// Reasoning behind the verification decision
    pub reasoning: String,

    /// Issues found if verification failed
    pub issues: Vec<String>,
}

/// A validated patch that passed verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedPatch {
    /// The verified patch
    pub patch: String,

    /// The location that was verified
    pub location: LocationCandidate,

    /// Number of attempts needed to find the correct location
    pub attempts: usize,
}

/// Coordinates the validation loop using Agent methods
///
/// Note: This is a simple struct for now. In the future, it could be used
/// to coordinate between separate Explorer and Builder agent instances if needed.
pub struct ValidationLoopCoordinator {
    /// Maximum number of attempts before giving up
    max_attempts: usize,
}

impl ValidationLoopCoordinator {
    /// Create a new validation loop coordinator
    pub fn new() -> Self {
        Self {
            max_attempts: DEFAULT_MAX_ATTEMPTS,
        }
    }

    /// Create a coordinator with custom max attempts
    pub fn with_max_attempts(max_attempts: usize) -> Self {
        Self {
            max_attempts,
        }
    }

    /// Get the maximum number of attempts
    pub fn max_attempts(&self) -> usize {
        self.max_attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_candidate_serialization() {
        let candidate = LocationCandidate {
            file_path: PathBuf::from("src/auth.rs"),
            line_number: Some(42),
            confidence: 0.95,
            reasoning: "Variable name matches bug description".to_string(),
        };

        let json = serde_json::to_string(&candidate).unwrap();
        let deserialized: LocationCandidate = serde_json::from_str(&json).unwrap();

        assert_eq!(candidate.file_path, deserialized.file_path);
        assert_eq!(candidate.line_number, deserialized.line_number);
        assert!((candidate.confidence - deserialized.confidence).abs() < 0.001);
    }

    #[test]
    fn test_patch_proposal_serialization() {
        let proposal = PatchProposal {
            location: LocationCandidate {
                file_path: PathBuf::from("src/auth.rs"),
                line_number: Some(42),
                confidence: 0.95,
                reasoning: "Test".to_string(),
            },
            patch: "--- a/src/auth.rs\n+++ b/src/auth.rs\n@@ -42,1 +42,1 @@\n-    old line\n+    new line".to_string(),
            reasoning: "Fix authentication bug".to_string(),
        };

        let json = serde_json::to_string(&proposal).unwrap();
        let deserialized: PatchProposal = serde_json::from_str(&json).unwrap();

        assert_eq!(proposal.location.file_path, deserialized.location.file_path);
        assert_eq!(proposal.patch, deserialized.patch);
    }

    #[test]
    fn test_verification_result_serialization() {
        let result = VerificationResult {
            is_correct: false,
            reasoning: "File doesn't contain expected code".to_string(),
            issues: vec![
                "Variable X not found".to_string(),
                "Line is in documentation, not implementation".to_string(),
            ],
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: VerificationResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.is_correct, deserialized.is_correct);
        assert_eq!(result.issues.len(), deserialized.issues.len());
    }

    #[test]
    fn test_validated_patch_serialization() {
        let validated = ValidatedPatch {
            patch: "--- a/file\n+++ b/file\n@@ -1,1 +1,1 @@\n-old\n+new".to_string(),
            location: LocationCandidate {
                file_path: PathBuf::from("test.rs"),
                line_number: Some(10),
                confidence: 0.9,
                reasoning: "Test".to_string(),
            },
            attempts: 2,
        };

        let json = serde_json::to_string(&validated).unwrap();
        let deserialized: ValidatedPatch = serde_json::from_str(&json).unwrap();

        assert_eq!(validated.attempts, deserialized.attempts);
        assert_eq!(validated.location.file_path, deserialized.location.file_path);
    }
}
