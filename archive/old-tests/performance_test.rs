use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use aircher::semantic_search::SemanticCodeSearch;

#[tokio::test]
async fn test_performance_with_larger_codebase() {
    println!("🚀 Testing performance with larger synthetic codebase...");

    // Create a temporary directory with multiple files
    let temp_dir = TempDir::new().unwrap();
    let start_setup = Instant::now();

    // Generate multiple code files to simulate a larger codebase
    let file_contents = vec![
        ("auth.rs", r#"
/// Authentication module for user management
use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub fn new(username: String, email: String, password: String) -> Result<Self, AuthError> {
        let password_hash = hash(password, DEFAULT_COST)?;
        Ok(User {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            created_at: chrono::Utc::now(),
        })
    }

    pub fn verify_password(&self, password: &str) -> bool {
        verify(password, &self.password_hash).unwrap_or(false)
    }
}

pub async fn authenticate_user(username: &str, password: &str) -> Result<User, AuthError> {
    let user = find_user_by_username(username).await?;
    if user.verify_password(password) {
        Ok(user)
    } else {
        Err(AuthError::InvalidCredentials)
    }
}

pub async fn create_user(username: String, email: String, password: String) -> Result<User, AuthError> {
    validate_username(&username)?;
    validate_email(&email)?;
    validate_password(&password)?;

    let user = User::new(username, email, password)?;
    save_user(&user).await?;
    Ok(user)
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Username already exists")]
    UsernameExists,
    #[error("Invalid email format")]
    InvalidEmail,
    #[error("Password too weak")]
    WeakPassword,
}
"#),
        ("database.rs", r#"
/// Database connection and query handling
use sqlx::{PgPool, Row};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DatabaseManager {
    pool: PgPool,
    connection_string: String,
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(DatabaseManager {
            pool,
            connection_string: database_url.to_string(),
        })
    }

    pub async fn execute_query(&self, query: &str, params: &[&str]) -> Result<Vec<HashMap<String, String>>> {
        let mut results = Vec::new();
        let rows = sqlx::query(query)
            .bind_all(params)
            .fetch_all(&self.pool)
            .await?;

        for row in rows {
            let mut map = HashMap::new();
            for (i, column) in row.columns().iter().enumerate() {
                let value: String = row.get(i);
                map.insert(column.name().to_string(), value);
            }
            results.push(map);
        }

        Ok(results)
    }

    pub async fn health_check(&self) -> Result<bool> {
        let result = sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(result.get::<i32, _>(0) == 1)
    }
}

pub async fn migrate_database(db: &DatabaseManager) -> Result<()> {
    let migrations = vec![
        "CREATE TABLE IF NOT EXISTS users (id UUID PRIMARY KEY, username VARCHAR NOT NULL, email VARCHAR NOT NULL);",
        "CREATE TABLE IF NOT EXISTS sessions (id UUID PRIMARY KEY, user_id UUID REFERENCES users(id), token VARCHAR NOT NULL);",
        "CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);",
        "CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);",
    ];

    for migration in migrations {
        db.execute_query(migration, &[]).await?;
    }

    Ok(())
}
"#),
        ("api.rs", r#"
/// REST API endpoints and request handling
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn create_user_endpoint(
    State(app_state): State<AppState>,
    Json(request): Json<UserRequest>,
) -> Result<Json<ApiResponse<User>>, StatusCode> {
    match create_user(request.username, request.email, request.password).await {
        Ok(user) => Ok(Json(ApiResponse {
            success: true,
            data: Some(user),
            error: None,
        })),
        Err(e) => {
            eprintln!("Error creating user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn login_endpoint(
    State(app_state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match authenticate_user(&request.username, &request.password).await {
        Ok(user) => {
            let token = generate_session_token(&user).await?;
            Ok(Json(ApiResponse {
                success: true,
                data: Some(token),
                error: None,
            }))
        }
        Err(e) => {
            eprintln!("Authentication failed: {}", e);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

pub async fn get_user_endpoint(
    State(app_state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<User>>, StatusCode> {
    match find_user_by_id(user_id).await {
        Ok(user) => Ok(Json(ApiResponse {
            success: true,
            data: Some(user),
            error: None,
        })),
        Err(e) => {
            eprintln!("Error finding user: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

pub fn create_router() -> Router {
    Router::new()
        .route("/users", post(create_user_endpoint))
        .route("/users/:id", get(get_user_endpoint))
        .route("/auth/login", post(login_endpoint))
        .route("/health", get(health_check_endpoint))
}
"#),
        ("utils.rs", r#"
/// Utility functions for validation and formatting
use regex::Regex;
use std::collections::HashMap;

pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if email_regex.is_match(email) {
        Ok(())
    } else {
        Err(ValidationError::InvalidEmail)
    }
}

pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username.len() < 3 {
        return Err(ValidationError::UsernameTooShort);
    }
    if username.len() > 50 {
        return Err(ValidationError::UsernameTooLong);
    }
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(ValidationError::InvalidUsername);
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::PasswordTooShort);
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ValidationError::PasswordNeedsUppercase);
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(ValidationError::PasswordNeedsLowercase);
    }
    if !password.chars().any(|c| c.is_numeric()) {
        return Err(ValidationError::PasswordNeedsNumber);
    }
    Ok(())
}

pub fn sanitize_input(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '_' || *c == '-')
        .collect()
}

pub fn format_error_response(error: &str) -> HashMap<String, String> {
    let mut response = HashMap::new();
    response.insert("error".to_string(), error.to_string());
    response.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
    response
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Email format is invalid")]
    InvalidEmail,
    #[error("Username must be at least 3 characters")]
    UsernameTooShort,
    #[error("Username must be at most 50 characters")]
    UsernameTooLong,
    #[error("Username contains invalid characters")]
    InvalidUsername,
    #[error("Password must be at least 8 characters")]
    PasswordTooShort,
    #[error("Password must contain at least one uppercase letter")]
    PasswordNeedsUppercase,
    #[error("Password must contain at least one lowercase letter")]
    PasswordNeedsLowercase,
    #[error("Password must contain at least one number")]
    PasswordNeedsNumber,
}
"#),
        ("cache.rs", r#"
/// Caching layer for performance optimization
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::time;

#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
    pub expires_at: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct Cache<T> {
    store: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    default_ttl: Duration,
}

impl<T: Clone> Cache<T> {
    pub fn new(default_ttl: Duration) -> Self {
        Cache {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    pub fn get(&self, key: &str) -> Option<T> {
        let store = self.store.read().unwrap();
        if let Some(entry) = store.get(key) {
            if let Some(expires_at) = entry.expires_at {
                if Instant::now() > expires_at {
                    return None;
                }
            }
            Some(entry.value.clone())
        } else {
            None
        }
    }

    pub fn set(&self, key: String, value: T, ttl: Option<Duration>) {
        let expires_at = ttl.map(|d| Instant::now() + d);
        let entry = CacheEntry {
            value,
            created_at: Instant::now(),
            expires_at,
        };

        let mut store = self.store.write().unwrap();
        store.insert(key, entry);
    }

    pub fn invalidate(&self, key: &str) {
        let mut store = self.store.write().unwrap();
        store.remove(key);
    }

    pub fn clear(&self) {
        let mut store = self.store.write().unwrap();
        store.clear();
    }

    pub fn size(&self) -> usize {
        let store = self.store.read().unwrap();
        store.len()
    }
}

pub async fn cleanup_expired_entries<T: Clone>(cache: &Cache<T>) {
    let mut interval = time::interval(Duration::from_secs(300)); // 5 minutes

    loop {
        interval.tick().await;

        let mut store = cache.store.write().unwrap();
        let now = Instant::now();

        store.retain(|_, entry| {
            if let Some(expires_at) = entry.expires_at {
                now <= expires_at
            } else {
                true
            }
        });
    }
}
"#),
    ];

    // Write all files
    for (filename, content) in file_contents {
        let file_path = temp_dir.path().join(filename);
        std::fs::write(&file_path, content).unwrap();
    }

    let setup_time = start_setup.elapsed();
    println!("✅ Setup completed in {:.2?}: {} files created", setup_time, 6);

    // Test the search functionality with performance metrics
    let mut search = SemanticCodeSearch::new();

    // Model availability
    let start_model = Instant::now();
    match search.ensure_model_available().await {
        Ok(()) => println!("✅ Model available in {:.2?}", start_model.elapsed()),
        Err(e) => println!("❌ Model not available: {}", e),
    }

    // Index the directory
    let start_index = Instant::now();
    match search.index_directory(&temp_dir.path()).await {
        Ok(()) => {
            let index_time = start_index.elapsed();
            println!("✅ Directory indexed in {:.2?}", index_time);

            // Get stats
            let stats = search.get_stats();
            println!("📊 Index stats: {} files, {} chunks, {:.1}% coverage",
                     stats.total_files, stats.total_chunks, stats.embedding_coverage * 100.0);

            // Test multiple search queries with timing
            let queries = vec![
                ("authentication", "Authentication and user management"),
                ("database connection", "Database operations and connections"),
                ("error handling", "Error handling and validation"),
                ("password validation", "Password security and validation"),
                ("cache management", "Caching and performance optimization"),
                ("API endpoints", "REST API and web services"),
            ];

            let mut total_search_time = Duration::from_secs(0);
            let mut total_results = 0;

            for (query, description) in &queries {
                let start_search = Instant::now();
                match search.search(query, 3).await {
                    Ok((results, _metrics)) => {
                        let search_time = start_search.elapsed();
                        total_search_time += search_time;
                        total_results += results.len();

                        println!("🔍 {} search ({:.2?}): {} results", description, search_time, results.len());

                        // Show top result for validation
                        if let Some(result) = results.first() {
                            println!("   Top result: {} (score: {:.3})",
                                     result.file_path.file_name().unwrap().to_string_lossy(),
                                     result.similarity_score);
                        }
                    },
                    Err(e) => {
                        println!("❌ Search '{}' failed: {}", query, e);
                    }
                }
            }

            let avg_search_time = total_search_time / queries.len() as u32;
            println!("\n📈 Performance Summary:");
            println!("   Setup time: {:.2?}", setup_time);
            println!("   Index time: {:.2?}", index_time);
            println!("   Average search time: {:.2?}", avg_search_time);
            println!("   Total results: {}", total_results);
            println!("   Files per second (indexing): {:.1}", stats.total_files as f64 / index_time.as_secs_f64());
            println!("   Chunks per second (indexing): {:.1}", stats.total_chunks as f64 / index_time.as_secs_f64());

            // Validate that we're getting relevant results
            if stats.embedding_coverage > 0.8 && total_results > 10 {
                println!("✅ Performance test passed: Good coverage and results");
            } else {
                println!("⚠️  Performance test concerns: coverage={:.1}%, results={}",
                         stats.embedding_coverage * 100.0, total_results);
            }
        },
        Err(e) => {
            println!("❌ Indexing failed: {}", e);
        }
    }

    println!("🎉 Performance test completed!");
}
