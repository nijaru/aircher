use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub role: MessageRole,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSession {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub title: String,
    pub entries: Vec<ConversationEntry>,
    pub metadata: HashMap<String, String>,
}

pub struct ConversationManager {
    storage_dir: PathBuf,
    sessions: HashMap<String, ConversationSession>,
}

impl ConversationManager {
    pub fn new(storage_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&storage_dir)?;
        Ok(Self {
            storage_dir,
            sessions: HashMap::new(),
        })
    }

    pub async fn create_session(&mut self, title: Option<String>) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let session = ConversationSession {
            id: session_id.clone(),
            created_at: now,
            updated_at: now,
            title: title.unwrap_or_else(|| format!("Conversation {}", now.format("%Y-%m-%d %H:%M"))),
            entries: Vec::new(),
            metadata: HashMap::new(),
        };

        self.sessions.insert(session_id.clone(), session);
        self.save_session(&session_id).await?;

        Ok(session_id)
    }

    pub async fn add_message(
        &mut self,
        session_id: &str,
        role: MessageRole,
        content: String,
        tool_calls: Option<Vec<ToolCall>>,
    ) -> Result<()> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        let entry = ConversationEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            role,
            content,
            tool_calls,
            session_id: session_id.to_string(),
        };

        session.entries.push(entry);
        session.updated_at = Utc::now();

        self.save_session(session_id).await?;
        Ok(())
    }

    pub fn get_session(&self, session_id: &str) -> Option<&ConversationSession> {
        self.sessions.get(session_id)
    }

    pub fn get_recent_sessions(&self, limit: usize) -> Vec<&ConversationSession> {
        let mut sessions: Vec<_> = self.sessions.values().collect();
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        sessions.into_iter().take(limit).collect()
    }

    pub async fn load_session(&mut self, session_id: &str) -> Result<()> {
        let file_path = self.storage_dir.join(format!("{}.json", session_id));

        if !file_path.exists() {
            return Err(anyhow::anyhow!("Session file not found: {}", session_id));
        }

        let content = fs::read_to_string(&file_path).await?;
        let session: ConversationSession = serde_json::from_str(&content)?;

        self.sessions.insert(session_id.to_string(), session);
        Ok(())
    }

    pub async fn save_session(&self, session_id: &str) -> Result<()> {
        let session = self.sessions.get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        let file_path = self.storage_dir.join(format!("{}.json", session_id));
        let content = serde_json::to_string_pretty(session)?;

        fs::write(&file_path, content).await?;
        Ok(())
    }

    pub async fn list_sessions(&mut self) -> Result<Vec<String>> {
        let mut sessions = Vec::new();
        let mut entries = fs::read_dir(&self.storage_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    sessions.push(stem.to_string());
                }
            }
        }

        Ok(sessions)
    }

    pub async fn delete_session(&mut self, session_id: &str) -> Result<()> {
        let file_path = self.storage_dir.join(format!("{}.json", session_id));

        if file_path.exists() {
            fs::remove_file(&file_path).await?;
        }

        self.sessions.remove(session_id);
        Ok(())
    }

    pub fn search_conversations(&self, query: &str, limit: usize) -> Vec<&ConversationEntry> {
        let query_lower = query.to_lowercase();
        let mut matches = Vec::new();

        for session in self.sessions.values() {
            for entry in &session.entries {
                if entry.content.to_lowercase().contains(&query_lower) {
                    matches.push(entry);
                }
            }
        }

        matches.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        matches.into_iter().take(limit).collect()
    }

    pub fn get_conversation_context(&self, session_id: &str, last_n: usize) -> Vec<&ConversationEntry> {
        if let Some(session) = self.sessions.get(session_id) {
            session.entries.iter()
                .rev()
                .take(last_n)
                .rev()
                .collect()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_conversation_persistence() -> Result<()> {
        let temp_dir = tempdir()?;
        let mut manager = ConversationManager::new(temp_dir.path().to_path_buf())?;

        // Create a session
        let session_id = manager.create_session(Some("Test Conversation".to_string())).await?;

        // Add messages
        manager.add_message(
            &session_id,
            MessageRole::User,
            "Hello, how are you?".to_string(),
            None,
        ).await?;

        manager.add_message(
            &session_id,
            MessageRole::Assistant,
            "I'm doing well, thank you!".to_string(),
            None,
        ).await?;

        // Verify session exists and has messages
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.entries.len(), 2);
        assert_eq!(session.title, "Test Conversation");

        // Create new manager and load session
        let mut new_manager = ConversationManager::new(temp_dir.path().to_path_buf())?;
        new_manager.load_session(&session_id).await?;

        let loaded_session = new_manager.get_session(&session_id).unwrap();
        assert_eq!(loaded_session.entries.len(), 2);
        assert_eq!(loaded_session.entries[0].content, "Hello, how are you?");

        Ok(())
    }

    #[tokio::test]
    async fn test_conversation_search() -> Result<()> {
        let temp_dir = tempdir()?;
        let mut manager = ConversationManager::new(temp_dir.path().to_path_buf())?;

        let session_id = manager.create_session(None).await?;

        manager.add_message(
            &session_id,
            MessageRole::User,
            "Help me debug this authentication error".to_string(),
            None,
        ).await?;

        manager.add_message(
            &session_id,
            MessageRole::Assistant,
            "Sure, let's check your authentication configuration".to_string(),
            None,
        ).await?;

        // Search for authentication
        let results = manager.search_conversations("authentication", 5);
        assert_eq!(results.len(), 2);

        // Search for something that doesn't exist
        let no_results = manager.search_conversations("database", 5);
        assert_eq!(no_results.len(), 0);

        Ok(())
    }
}