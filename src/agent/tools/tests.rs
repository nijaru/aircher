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

    #[tokio::test]
    async fn test_analyze_code_tool_fallback() {
        // Test AnalyzeCodeTool without intelligence engine (fallback mode)
        let tool = code_analysis::AnalyzeCodeTool::new();

        let params = json!({
            "file_path": "Cargo.toml",
            "include_suggestions": true
        });

        let result = tool.execute(params).await.unwrap();
        assert!(result.success);

        // Check that analysis contains expected fields
        assert!(result.result["analysis"].is_object());
        assert!(result.result["analysis"]["file_path"].as_str().unwrap().contains("Cargo.toml"));
        assert_eq!(result.result["analysis"]["language"], "unknown"); // .toml not detected in fallback
        assert!(result.result["analysis"]["lines_of_code"].as_u64().unwrap() > 0);
        assert!(result.result["summary"].as_str().unwrap().contains("LOC"));
    }

    #[tokio::test]
    async fn test_analyze_code_tool_rust_file() {
        // Test analyzing a Rust source file
        let tool = code_analysis::AnalyzeCodeTool::new();

        // Use a known Rust file
        let params = json!({
            "file_path": "src/main.rs",
            "include_suggestions": true
        });

        let result = tool.execute(params).await.unwrap();
        assert!(result.success);

        // Check basic analysis
        let analysis = &result.result["analysis"];
        assert_eq!(analysis["language"], "rust");
        assert!(analysis["lines_of_code"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_analyze_code_tool_nonexistent_file() {
        // Test handling of nonexistent file
        let tool = code_analysis::AnalyzeCodeTool::new();

        let params = json!({
            "file_path": "/nonexistent/file/path.rs",
            "include_suggestions": true
        });

        let result = tool.execute(params).await.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.result["message"].as_str().unwrap().contains("not found") ||
                result.result["message"].as_str().unwrap().contains("not exist"));
    }

    #[tokio::test]
    async fn test_search_code_tool_without_engine() {
        // Test SearchCodeTool without intelligence engine (should return appropriate message)
        let tool = code_analysis::SearchCodeTool::new();

        let params = json!({
            "query": "semantic search",
            "limit": 5
        });

        let result = tool.execute(params).await.unwrap();
        assert!(result.success);
        assert_eq!(result.result["count"], 0);
        assert!(result.result["message"].as_str().unwrap().contains("not available") ||
                result.result["message"].as_str().unwrap().contains("needs proper integration"));
    }

    #[tokio::test]
    async fn test_find_definition_tool() {
        // Test FindDefinitionTool with ripgrep
        let tool = code_analysis::FindDefinitionTool::new();

        let params = json!({
            "symbol": "AgentTool",
            "file_types": ["rs"]
        });

        let result = tool.execute(params).await.unwrap();
        assert!(result.success);

        // Should find at least one definition (the trait definition)
        if let Some(definitions) = result.result["definitions"].as_array() {
            // May or may not find results depending on ripgrep availability
            // Just verify the structure is correct
            if !definitions.is_empty() {
                assert!(definitions[0]["file"].is_string());
                assert!(definitions[0]["line"].is_number());
                assert!(definitions[0]["text"].is_string());
            }
        }
    }
}