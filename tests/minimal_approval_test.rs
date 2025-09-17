use anyhow::Result;

#[test]
fn test_approval_modes_exist() -> Result<()> {
    use aircher::agent::approval_modes::ApprovalMode;

    // Test that approval modes exist and work
    let mode = ApprovalMode::Review;
    assert_eq!(mode, ApprovalMode::default());

    let smart_mode = ApprovalMode::Smart;
    assert_ne!(smart_mode, mode);

    println!("✅ ApprovalMode enum is accessible and functional");
    Ok(())
}

#[test]
fn test_approval_registry_exists() -> Result<()> {
    use aircher::agent::tools::approval_registry::create_agent_registry_with_approval;

    // Test that approval registry can be created
    let (registry, _receiver) = create_agent_registry_with_approval();

    // Verify it has tools
    let tools = registry.list_tools();
    assert!(!tools.is_empty(), "Registry should have tools");

    // Check for approved file tools
    let has_write_tool = tools.iter().any(|t| t.name == "write_file");
    assert!(has_write_tool, "Should have write_file tool");

    println!("✅ Approval registry creation works");
    println!("Registry has {} tools", tools.len());

    Ok(())
}

#[test]
fn test_pending_change_structure() -> Result<()> {
    use aircher::agent::approval_modes::{PendingChange, ChangeType};
    use std::path::PathBuf;

    // Test that PendingChange can be created
    let change = PendingChange {
        id: "test-123".to_string(),
        change_type: ChangeType::CreateFile {
            path: PathBuf::from("/tmp/test.rs"),
            content: "fn main() {}".to_string(),
        },
        description: "Test file creation".to_string(),
        safety_level: aircher::agent::approval_modes::SafetyLevel::Safe,
        timestamp: chrono::Utc::now(),
    };

    // Test diff generation
    let diff = change.generate_diff();
    assert!(!diff.is_empty(), "Diff should not be empty");
    assert!(diff.contains("fn main"), "Diff should contain the content");

    println!("✅ PendingChange structure works");
    println!("Generated diff: {} chars", diff.len());

    Ok(())
}