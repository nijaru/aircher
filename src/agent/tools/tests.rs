#[cfg(test)]
mod tests {
    use super::super::*;
    use serde_json::json;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    #[tokio::test]
    async fn test_read_file_tool() {
        let tool = file_ops::ReadFileTool::new();
        let params = json!({
            "path": "Cargo.toml"
        });
        
        let result = tool.execute(params).await.unwrap();
        assert!(result.success);
        assert!(result.result["content"].as_str().unwrap().contains("name = \"aircher\""));
    }
    
    #[tokio::test]
    async fn test_list_files_tool() {
        let tool = file_ops::ListFilesTool::new();
        let params = json!({
            "path": ".",
            "recursive": false
        });
        
        let result = tool.execute(params).await.unwrap();
        assert!(result.success);
        
        // Check that we got some results
        if let Some(files) = result.result["files"].as_array() {
            // Should have some files in the directory
            assert!(!files.is_empty());
        }
        if let Some(dirs) = result.result["directories"].as_array() {
            // Should have some directories too
            assert!(!dirs.is_empty());
        }
    }
    
    #[test]
    fn test_permissions_manager() {
        let mut perms = crate::permissions::PermissionsManager::new().unwrap();
        
        // Test pre-approved commands
        assert!(perms.is_command_approved("ls"));
        assert!(perms.is_command_approved("pwd"));
        assert!(perms.is_command_approved("echo hello"));
        
        // Test unapproved commands
        assert!(!perms.is_command_approved("rm -rf /"));
        assert!(!perms.is_command_approved("dangerous_command"));
        
        // Test approval
        perms.approve_command("cargo test".to_string()).unwrap();
        assert!(perms.is_command_approved("cargo test"));
        
        // Test pattern approval
        perms.approve_pattern("npm".to_string()).unwrap();
        assert!(perms.is_command_approved("npm install"));
        assert!(perms.is_command_approved("npm run test"));
    }
    
    #[tokio::test]
    async fn test_run_command_with_permissions() {
        let perms = crate::permissions::PermissionsManager::new().unwrap();
        let (tx, mut rx) = create_permission_channel();
        let perms_arc = Arc::new(Mutex::new(perms));
        
        let tool = system_ops::RunCommandTool::with_permissions(
            perms_arc.clone(),
            Some(tx)
        );
        
        // Test pre-approved command
        let params = json!({
            "command": "echo",
            "args": ["test"]
        });
        
        let result = tool.execute(params).await.unwrap();
        assert!(result.success);
        assert_eq!(result.result["stdout"].as_str().unwrap().trim(), "test");
        
        // Test permission channel is not used for pre-approved commands
        assert!(rx.try_recv().is_err());
    }
    
    #[test]
    fn test_tool_parser() {
        let parser = crate::agent::parser::ToolCallParser::new().unwrap();
        
        // Test XML style
        let content = "<tool>read_file</tool><params>{\"path\": \"test.txt\"}</params>";
        let calls = parser.parse(content);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "read_file");
        assert_eq!(calls[0].parameters["path"], "test.txt");
        
        // Test structured parsing
        let content = "Let me read the file.\n<tool_use>\n<tool>read_file</tool><params>{\"path\": \"README.md\"}</params>\n</tool_use>\nDone!";
        let (text, calls) = parser.parse_structured(content).unwrap();
        assert_eq!(calls.len(), 1);
        assert!(text.contains("Let me read the file"));
        assert!(text.contains("Done!"));
        assert!(!text.contains("<tool_use>"));
    }
}