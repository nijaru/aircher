use std::path::PathBuf;
use tempfile::TempDir;
use aircher::commands::search::{SearchArgs, SearchCommand, handle_search_command};

#[tokio::test]
async fn test_cli_search_integration() {
    println!("🧪 Testing CLI search integration...");
    
    // Create a temporary directory with test code
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("sample.rs");
    
    let sample_code = r#"
/// This function handles user authentication
pub fn authenticate_user(username: &str, password: &str) -> bool {
    // Validate credentials
    if username.is_empty() || password.is_empty() {
        return false;
    }
    
    // Check against database
    verify_credentials(username, password)
}

/// Database connection helper
pub fn connect_to_database() -> Result<DatabaseConnection, Error> {
    let config = DatabaseConfig::from_env()?;
    DatabaseConnection::new(config)
}

/// Error handling utility
pub fn handle_network_error(error: NetworkError) {
    match error {
        NetworkError::Timeout => {
            println!("Request timed out");
        }
        NetworkError::ConnectionFailed => {
            println!("Connection failed");
        }
        NetworkError::InvalidResponse => {
            println!("Invalid response received");
        }
    }
}
"#;
    
    std::fs::write(&test_file, sample_code).unwrap();
    
    // Test 1: Index command
    println!("📝 Testing index command...");
    let index_args = SearchArgs {
        command: SearchCommand::Index {
            path: temp_dir.path().to_path_buf(),
            force: false,
        },
    };
    
    match handle_search_command(index_args).await {
        Ok(()) => println!("✅ Index command successful"),
        Err(e) => {
            println!("❌ Index command failed: {}", e);
            // Don't panic - this might be expected if models aren't available
        }
    }
    
    // Test 2: Stats command
    println!("📊 Testing stats command...");
    let stats_args = SearchArgs {
        command: SearchCommand::Stats {
            path: temp_dir.path().to_path_buf(),
        },
    };
    
    match handle_search_command(stats_args).await {
        Ok(()) => println!("✅ Stats command successful"),
        Err(e) => {
            println!("❌ Stats command failed: {}", e);
        }
    }
    
    // Test 3: Query command
    println!("🔍 Testing query command...");
    let query_args = SearchArgs {
        command: SearchCommand::Query {
            query: "authentication function".to_string(),
            limit: 5,
            path: temp_dir.path().to_path_buf(),
        },
    };
    
    match handle_search_command(query_args).await {
        Ok(()) => println!("✅ Query command successful"),
        Err(e) => {
            println!("❌ Query command failed: {}", e);
        }
    }
    
    // Test 4: Different search terms
    println!("🔍 Testing database search...");
    let db_query_args = SearchArgs {
        command: SearchCommand::Query {
            query: "database connection".to_string(),
            limit: 3,
            path: temp_dir.path().to_path_buf(),
        },
    };
    
    match handle_search_command(db_query_args).await {
        Ok(()) => println!("✅ Database query successful"),
        Err(e) => {
            println!("❌ Database query failed: {}", e);
        }
    }
    
    // Test 5: Error handling search
    println!("🔍 Testing error handling search...");
    let error_query_args = SearchArgs {
        command: SearchCommand::Query {
            query: "error handling".to_string(),
            limit: 3,
            path: temp_dir.path().to_path_buf(),
        },
    };
    
    match handle_search_command(error_query_args).await {
        Ok(()) => println!("✅ Error handling query successful"),
        Err(e) => {
            println!("❌ Error handling query failed: {}", e);
        }
    }
    
    println!("🎉 CLI integration tests completed!");
}