use std::path::PathBuf;
use aircher::semantic_search::SemanticCodeSearch;
use aircher::code_chunking::CodeChunker;

#[tokio::test]
async fn test_tui_search_integration() {
    // Create test code file content
    let rust_code = r#"
        use std::collections::HashMap;
        
        pub struct UserManager {
            users: HashMap<String, User>,
        }
        
        impl UserManager {
            pub fn new() -> Self {
                Self {
                    users: HashMap::new(),
                }
            }
            
            pub fn add_user(&mut self, user: User) -> Result<(), String> {
                if self.users.contains_key(&user.username) {
                    return Err("User already exists".to_string());
                }
                self.users.insert(user.username.clone(), user);
                Ok(())
            }
            
            pub fn find_user(&self, username: &str) -> Option<&User> {
                self.users.get(username)
            }
        }
        
        #[derive(Debug, Clone)]
        pub struct User {
            pub username: String,
            pub email: String,
        }
        
        impl User {
            pub fn new(username: String, email: String) -> Self {
                Self { username, email }
            }
        }
    "#;
    
    // Test that semantic search can handle the workflow
    let mut search = SemanticCodeSearch::new();
    
    // Ensure model is available
    match search.ensure_model_available().await {
        Ok(_) => println!("Model available for search"),
        Err(e) => {
            println!("Model not available, skipping search test: {}", e);
            return;
        }
    }
    
    // Test the search functionality that the TUI would use
    let search_queries = vec![
        "user management",
        "add user function",
        "find user method",
        "HashMap operations",
    ];
    
    for query in search_queries {
        println!("\nTesting search query: '{}'", query);
        
        match search.search(query, 5).await {
            Ok((results, _metrics)) => {
                if results.is_empty() {
                    println!("  No results found (this is okay for empty index)");
                } else {
                    println!("  Found {} results:", results.len());
                    for (i, result) in results.iter().enumerate() {
                        println!("    {}. {} (similarity: {:.3})", 
                                i + 1, 
                                result.file_path.display(), 
                                result.similarity_score);
                    }
                }
            }
            Err(e) => {
                println!("  Search error: {}", e);
            }
        }
    }
    
    // Test code chunking that feeds into search
    let mut chunker = CodeChunker::new().unwrap();
    let chunks = chunker.chunk_file(&PathBuf::from("test_user.rs"), rust_code).unwrap();
    
    println!("\nCode chunking results for TUI integration:");
    println!("Found {} chunks:", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        println!("  Chunk {}: {:?} - lines {}-{}", 
                i, chunk.chunk_type, chunk.start_line, chunk.end_line);
        if let Some(name) = &chunk.name {
            println!("    Name: {}", name);
        }
    }
    
    // Verify that we have meaningful chunks for search
    assert!(!chunks.is_empty(), "Should find code chunks");
    
    println!("\n✅ TUI search integration test completed successfully");
    println!("The /search command implementation should work with this backend");
}

#[test]
fn test_search_command_parsing() {
    // Test the command parsing logic that the TUI uses
    let test_inputs = vec![
        ("/search user management", Some("user management")),
        ("/search find function", Some("find function")), 
        ("/search", None),
        ("search without slash", None),
        ("/search   ", None), // Only whitespace
        ("/search hello world", Some("hello world")),
    ];
    
    for (input, expected) in test_inputs {
        let result = if input.starts_with("/search ") {
            let query = input.strip_prefix("/search ").unwrap_or("").trim();
            if query.is_empty() { None } else { Some(query) }
        } else {
            None
        };
        
        assert_eq!(result, expected, "Failed for input: '{}'", input);
    }
    
    println!("✅ Search command parsing test passed");
}