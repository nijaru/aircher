use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use uuid::Uuid;

// Re-define MessageRole to avoid circular dependency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Simple Message type for sessions to avoid circular dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub tokens_used: Option<u32>,
    pub cost: Option<f64>,
}

use crate::storage::DatabaseManager;
use async_trait::async_trait;

/// Represents a conversation session with metadata and persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub provider: String,
    pub model: String,
    pub total_cost: f64,
    pub total_tokens: u32,
    pub message_count: u32,
    pub tags: Vec<String>,
    pub is_archived: bool,
    pub description: Option<String>,
}

/// Represents a message within a session with full metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub id: String,
    pub session_id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub tokens_used: Option<u32>,
    pub cost: Option<f64>,
    pub provider: String,
    pub model: String,
    pub finish_reason: Option<String>,
    pub sequence_number: i32,
}

/// Session search and filtering criteria
#[derive(Debug, Clone, Default)]
pub struct SessionFilter {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub tags: Vec<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub min_cost: Option<f64>,
    pub max_cost: Option<f64>,
    pub archived: Option<bool>,
    pub search_text: Option<String>,
}

/// Session analytics and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAnalytics {
    pub total_sessions: u32,
    pub total_messages: u32,
    pub total_cost: f64,
    pub total_tokens: u64,
    pub avg_session_cost: f64,
    pub avg_session_length: f32,
    pub provider_breakdown: HashMap<String, ProviderStats>,
    pub monthly_usage: Vec<MonthlyStats>,
    pub most_used_models: Vec<ModelUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStats {
    pub sessions: u32,
    pub messages: u32,
    pub cost: f64,
    pub tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyStats {
    pub year: i32,
    pub month: u32,
    pub sessions: u32,
    pub cost: f64,
    pub tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub provider: String,
    pub model: String,
    pub usage_count: u32,
    pub total_cost: f64,
}

/// Session export formats
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Markdown,
    Csv,
    Plain,
}

/// Main session manager for persistence and operations
pub struct SessionManager {
    db_pool: SqlitePool,
}

impl SessionManager {
    /// Create a new session manager with database connection
    pub async fn new(database_manager: &DatabaseManager) -> Result<Self> {
        let db_path = database_manager.get_sessions_db_path();

        let connection_string = if db_path.to_string_lossy() == ":memory:" {
            "sqlite::memory:".to_string()
        } else {
            // Ensure parent directory exists
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            format!("sqlite://{}?mode=rwc", db_path.display())
        };

        let pool = SqlitePool::connect(&connection_string).await?;

        let session_manager = Self { db_pool: pool };
        session_manager.initialize_schema().await?;

        Ok(session_manager)
    }

    /// Initialize database schema for sessions and messages
    async fn initialize_schema(&self) -> Result<()> {
        // Sessions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                total_cost REAL NOT NULL DEFAULT 0.0,
                total_tokens INTEGER NOT NULL DEFAULT 0,
                message_count INTEGER NOT NULL DEFAULT 0,
                tags TEXT NOT NULL DEFAULT '[]',
                is_archived BOOLEAN NOT NULL DEFAULT FALSE,
                description TEXT
            )
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        // Messages table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS session_messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                tokens_used INTEGER,
                cost REAL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                finish_reason TEXT,
                sequence_number INTEGER NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions (id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        // Create indexes for better query performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_provider ON sessions(provider)")
            .execute(&self.db_pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_created_at ON sessions(created_at)")
            .execute(&self.db_pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_messages_session_id ON session_messages(session_id)")
            .execute(&self.db_pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON session_messages(timestamp)")
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        title: String,
        provider: String,
        model: String,
        description: Option<String>,
        tags: Vec<String>,
    ) -> Result<Session> {
        let session = Session {
            id: Uuid::new_v4().to_string(),
            title,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            provider,
            model,
            total_cost: 0.0,
            total_tokens: 0,
            message_count: 0,
            tags,
            is_archived: false,
            description,
        };

        self.save_session(&session).await?;
        Ok(session)
    }

    /// Save or update a session
    pub async fn save_session(&self, session: &Session) -> Result<()> {
        let tags_json = serde_json::to_string(&session.tags)?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO sessions
            (id, title, created_at, updated_at, provider, model, total_cost,
             total_tokens, message_count, tags, is_archived, description)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&session.id)
        .bind(&session.title)
        .bind(session.created_at.to_rfc3339())
        .bind(session.updated_at.to_rfc3339())
        .bind(&session.provider)
        .bind(&session.model)
        .bind(session.total_cost)
        .bind(session.total_tokens as i64)
        .bind(session.message_count as i64)
        .bind(tags_json)
        .bind(session.is_archived)
        .bind(&session.description)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Load a session by ID
    pub async fn load_session(&self, session_id: &str) -> Result<Option<Session>> {
        let row = sqlx::query("SELECT * FROM sessions WHERE id = ?")
            .bind(session_id)
            .fetch_optional(&self.db_pool)
            .await?;

        if let Some(row) = row {
            let tags: Vec<String> = serde_json::from_str(row.get("tags"))?;

            Ok(Some(Session {
                id: row.get("id"),
                title: row.get("title"),
                created_at: DateTime::parse_from_rfc3339(row.get("created_at"))?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(row.get("updated_at"))?.with_timezone(&Utc),
                provider: row.get("provider"),
                model: row.get("model"),
                total_cost: row.get("total_cost"),
                total_tokens: row.get::<i64, _>("total_tokens") as u32,
                message_count: row.get::<i64, _>("message_count") as u32,
                tags,
                is_archived: row.get("is_archived"),
                description: row.get("description"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Add a message to a session
    pub async fn add_message(&self, session_id: &str, message: &Message) -> Result<()> {
        // Get current sequence number
        let sequence_number: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(sequence_number), 0) + 1 FROM session_messages WHERE session_id = ?"
        )
        .bind(session_id)
        .fetch_one(&self.db_pool)
        .await?;

        // Insert message
        sqlx::query(
            r#"
            INSERT INTO session_messages
            (id, session_id, role, content, timestamp, tokens_used, cost,
             provider, model, finish_reason, sequence_number)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&message.id)
        .bind(session_id)
        .bind(serde_json::to_string(&message.role)?.trim_matches('"'))
        .bind(&message.content)
        .bind(message.timestamp.to_rfc3339())
        .bind(message.tokens_used.map(|t| t as i64))
        .bind(message.cost)
        .bind("") // provider - we'll need to track this separately
        .bind("") // model - we'll need to track this separately
        .bind(None::<String>) // finish_reason
        .bind(sequence_number)
        .execute(&self.db_pool)
        .await?;

        // Update session statistics
        self.update_session_stats(session_id).await?;

        Ok(())
    }

    /// Update session statistics based on messages
    async fn update_session_stats(&self, session_id: &str) -> Result<()> {
        let stats: (i64, Option<f64>, Option<i64>) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as message_count,
                SUM(cost) as total_cost,
                SUM(tokens_used) as total_tokens
            FROM session_messages
            WHERE session_id = ?
            "#,
        )
        .bind(session_id)
        .fetch_one(&self.db_pool)
        .await?;

        sqlx::query(
            r#"
            UPDATE sessions
            SET message_count = ?, total_cost = ?, total_tokens = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(stats.0)
        .bind(stats.1.unwrap_or(0.0))
        .bind(stats.2.unwrap_or(0))
        .bind(Utc::now().to_rfc3339())
        .bind(session_id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Load all messages for a session
    pub async fn load_session_messages(&self, session_id: &str) -> Result<Vec<SessionMessage>> {
        let rows = sqlx::query("SELECT * FROM session_messages WHERE session_id = ? ORDER BY sequence_number ASC")
            .bind(session_id)
            .fetch_all(&self.db_pool)
            .await?;

        let mut messages = Vec::new();
        for row in rows {
            let role_str: String = row.get("role");
            let role = serde_json::from_str(&format!("\"{}\"", role_str))?;

            messages.push(SessionMessage {
                id: row.get("id"),
                session_id: row.get("session_id"),
                role,
                content: row.get("content"),
                timestamp: DateTime::parse_from_rfc3339(row.get("timestamp"))?.with_timezone(&Utc),
                tokens_used: row.get::<Option<i64>, _>("tokens_used").map(|t| t as u32),
                cost: row.get("cost"),
                provider: row.get("provider"),
                model: row.get("model"),
                finish_reason: row.get("finish_reason"),
                sequence_number: row.get::<i64, _>("sequence_number") as i32,
            });
        }

        Ok(messages)
    }

    /// Search sessions with filters
    pub async fn search_sessions(&self, filter: &SessionFilter, _limit: Option<u32>) -> Result<Vec<Session>> {
        // Build a simple query based on available filters
        let base_query = "SELECT * FROM sessions WHERE 1=1";

        let rows = if let Some(provider) = &filter.provider {
            sqlx::query(&format!("{} AND provider = ? ORDER BY updated_at DESC LIMIT 100", base_query))
                .bind(provider)
                .fetch_all(&self.db_pool)
                .await?
        } else {
            sqlx::query(&format!("{} ORDER BY updated_at DESC LIMIT 100", base_query))
                .fetch_all(&self.db_pool)
                .await?
        };

        let mut sessions = Vec::new();
        for row in rows {
            let tags: Vec<String> = serde_json::from_str(row.get("tags"))?;

            sessions.push(Session {
                id: row.get("id"),
                title: row.get("title"),
                created_at: DateTime::parse_from_rfc3339(row.get("created_at"))?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(row.get("updated_at"))?.with_timezone(&Utc),
                provider: row.get("provider"),
                model: row.get("model"),
                total_cost: row.get("total_cost"),
                total_tokens: row.get::<i64, _>("total_tokens") as u32,
                message_count: row.get::<i64, _>("message_count") as u32,
                tags,
                is_archived: row.get("is_archived"),
                description: row.get("description"),
            });
        }

        Ok(sessions)
    }

    /// Delete a session and all its messages
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(session_id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }

    /// Archive/unarchive a session
    pub async fn archive_session(&self, session_id: &str, archived: bool) -> Result<()> {
        sqlx::query("UPDATE sessions SET is_archived = ?, updated_at = ? WHERE id = ?")
            .bind(archived)
            .bind(Utc::now().to_rfc3339())
            .bind(session_id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }

    /// Get session analytics
    pub async fn get_analytics(&self) -> Result<SessionAnalytics> {
        // Total counts
        let totals: (i64, i64, Option<f64>) = sqlx::query_as(
            "SELECT COUNT(*), SUM(message_count), SUM(total_cost) FROM sessions"
        )
        .fetch_one(&self.db_pool)
        .await?;

        let total_tokens: Option<i64> = sqlx::query_scalar(
            "SELECT SUM(total_tokens) FROM sessions"
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(SessionAnalytics {
            total_sessions: totals.0 as u32,
            total_messages: totals.1 as u32,
            total_cost: totals.2.unwrap_or(0.0),
            total_tokens: total_tokens.unwrap_or(0) as u64,
            avg_session_cost: if totals.0 > 0 { totals.2.unwrap_or(0.0) / totals.0 as f64 } else { 0.0 },
            avg_session_length: if totals.0 > 0 { totals.1 as f32 / totals.0 as f32 } else { 0.0 },
            provider_breakdown: HashMap::new(), // TODO: Implement detailed breakdown
            monthly_usage: Vec::new(), // TODO: Implement monthly stats
            most_used_models: Vec::new(), // TODO: Implement model usage stats
        })
    }

    /// Export session to different formats
    pub async fn export_session(&self, session_id: &str, format: ExportFormat) -> Result<String> {
        let session = self.load_session(session_id).await?
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
        let messages = self.load_session_messages(session_id).await?;

        match format {
            ExportFormat::Json => {
                let export_data = serde_json::json!({
                    "session": session,
                    "messages": messages
                });
                Ok(serde_json::to_string_pretty(&export_data)?)
            }
            ExportFormat::Markdown => {
                let mut output = String::new();
                output.push_str(&format!("# {}\n\n", session.title));
                output.push_str(&format!("**Provider:** {}\n", session.provider));
                output.push_str(&format!("**Model:** {}\n", session.model));
                output.push_str(&format!("**Created:** {}\n", session.created_at.format("%Y-%m-%d %H:%M:%S")));
                output.push_str(&format!("**Total Cost:** ${:.4}\n", session.total_cost));
                output.push_str(&format!("**Total Tokens:** {}\n\n", session.total_tokens));

                if let Some(desc) = &session.description {
                    output.push_str(&format!("**Description:** {}\n\n", desc));
                }

                output.push_str("## Conversation\n\n");

                for message in messages {
                    let role_name = match message.role {
                        MessageRole::User => "User",
                        MessageRole::Assistant => "Assistant",
                        MessageRole::System => "System",
                        MessageRole::Tool => "Tool",
                    };

                    output.push_str(&format!("### {} ({})\n\n", role_name, message.timestamp.format("%H:%M:%S")));
                    output.push_str(&format!("{}\n\n", message.content));

                    if let Some(cost) = message.cost {
                        output.push_str(&format!("*Cost: ${:.4}*\n\n", cost));
                    }
                }

                Ok(output)
            }
            ExportFormat::Plain => {
                let mut output = String::new();
                output.push_str(&format!("{}\n", session.title));
                output.push_str(&format!("Provider: {} | Model: {}\n", session.provider, session.model));
                output.push_str(&format!("Created: {} | Cost: ${:.4}\n\n",
                    session.created_at.format("%Y-%m-%d %H:%M:%S"), session.total_cost));

                for message in messages {
                    let role_name = match message.role {
                        MessageRole::User => "USER",
                        MessageRole::Assistant => "ASSISTANT",
                        MessageRole::System => "SYSTEM",
                        MessageRole::Tool => "TOOL",
                    };

                    output.push_str(&format!("[{}] {}\n\n", role_name, message.content));
                }

                Ok(output)
            }
            ExportFormat::Csv => {
                let mut output = String::new();
                output.push_str("timestamp,role,content,tokens,cost\n");

                for message in messages {
                    let role_str = match message.role {
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant",
                        MessageRole::System => "system",
                        MessageRole::Tool => "tool",
                    };

                    let content = message.content.replace("\"", "\"\""); // Escape quotes
                    output.push_str(&format!("{},{},\"{}\",{},{}\n",
                        message.timestamp.to_rfc3339(),
                        role_str,
                        content,
                        message.tokens_used.unwrap_or(0),
                        message.cost.unwrap_or(0.0)
                    ));
                }

                Ok(output)
            }
        }
    }
}

/// Trait abstraction for SessionManager to enable testing
#[async_trait]
pub trait SessionManagerTrait: Send + Sync {
    async fn create_session(
        &self,
        title: String,
        provider: String,
        model: String,
        description: Option<String>,
        tags: Vec<String>,
    ) -> Result<Session>;

    async fn load_session(&self, session_id: &str) -> Result<Option<Session>>;
    async fn add_message(&self, session_id: &str, message: &Message) -> Result<()>;
    async fn load_session_messages(&self, session_id: &str) -> Result<Vec<SessionMessage>>;
    async fn save_session(&self, session: &Session) -> Result<()>;
}

/// Implement the trait for the real SessionManager
#[async_trait]
impl SessionManagerTrait for SessionManager {
    async fn create_session(
        &self,
        title: String,
        provider: String,
        model: String,
        description: Option<String>,
        tags: Vec<String>,
    ) -> Result<Session> {
        self.create_session(title, provider, model, description, tags).await
    }

    async fn load_session(&self, session_id: &str) -> Result<Option<Session>> {
        self.load_session(session_id).await
    }

    async fn add_message(&self, session_id: &str, message: &Message) -> Result<()> {
        self.add_message(session_id, message).await
    }

    async fn load_session_messages(&self, session_id: &str) -> Result<Vec<SessionMessage>> {
        self.load_session_messages(session_id).await
    }

    async fn save_session(&self, session: &Session) -> Result<()> {
        self.save_session(session).await
    }
}
