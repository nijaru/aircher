use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::agent::conversation::CodingConversation;
use crate::utils::aircher_dirs::{AircherDirs, AircherFileType};

/// Conversation metadata for listing and sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub is_shared: bool,
    pub share_url: Option<String>,
    pub tags: Vec<String>,
}

/// Shareable conversation export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareableConversation {
    pub metadata: ConversationMetadata,
    pub conversation: CodingConversation,
    pub export_timestamp: DateTime<Utc>,
    pub aircher_version: String,
}

/// Conversation storage manager
#[derive(Debug)]
pub struct ConversationStorage {
    storage_dir: PathBuf,
    index_file: PathBuf,
    conversations: HashMap<String, ConversationMetadata>,
}

impl ConversationStorage {
    pub fn new() -> Result<Self> {
        let storage_dir = AircherDirs::get_file_path(AircherFileType::Data, "conversations")?;
        Self::with_storage_dir(storage_dir)
    }

    pub fn with_storage_dir(storage_dir: PathBuf) -> Result<Self> {
        let index_file = storage_dir.join("index.json");

        // Create storage directory if it doesn't exist
        AircherDirs::ensure_dir_exists(&storage_dir)?;

        let conversations = Self::load_index(&index_file)?;

        Ok(Self {
            storage_dir,
            index_file,
            conversations,
        })
    }
    
    /// Save a conversation with metadata
    pub async fn save_conversation(
        &mut self,
        conversation: &CodingConversation,
        title: Option<String>,
        provider: Option<String>,
        model: Option<String>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        // Generate title from first user message if not provided
        let title = title.unwrap_or_else(|| {
            conversation.messages
                .iter()
                .find(|m| matches!(m.role, crate::agent::conversation::MessageRole::User))
                .map(|m| {
                    let content = m.content.trim();
                    if content.len() > 50 {
                        format!("{}...", &content[..47])
                    } else {
                        content.to_string()
                    }
                })
                .unwrap_or_else(|| format!("Conversation {}", now.format("%Y-%m-%d %H:%M")))
        });
        
        let metadata = ConversationMetadata {
            id: id.clone(),
            title,
            created_at: now,
            updated_at: now,
            message_count: conversation.messages.len(),
            provider,
            model,
            is_shared: false,
            share_url: None,
            tags: Vec::new(),
        };
        
        // Save conversation file
        let conversation_file = self.storage_dir.join(format!("{}.json", id));
        let content = serde_json::to_string_pretty(conversation)
            .context("Failed to serialize conversation")?;
        fs::write(&conversation_file, content)
            .context("Failed to write conversation file")?;
        
        // Update index
        self.conversations.insert(id.clone(), metadata);
        self.save_index()?;
        
        info!("Saved conversation: {} ({})", self.conversations[&id].title, id);
        Ok(id)
    }
    
    /// Load a conversation by ID
    pub async fn load_conversation(&self, id: &str) -> Result<CodingConversation> {
        let conversation_file = self.storage_dir.join(format!("{}.json", id));
        if !conversation_file.exists() {
            return Err(anyhow::anyhow!("Conversation {} not found", id));
        }
        
        let content = fs::read_to_string(&conversation_file)
            .context("Failed to read conversation file")?;
        let conversation: CodingConversation = serde_json::from_str(&content)
            .context("Failed to parse conversation file")?;
        
        debug!("Loaded conversation: {}", id);
        Ok(conversation)
    }
    
    /// List all conversations
    pub fn list_conversations(&self) -> Vec<&ConversationMetadata> {
        let mut conversations: Vec<_> = self.conversations.values().collect();
        conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        conversations
    }
    
    /// Delete a conversation
    pub async fn delete_conversation(&mut self, id: &str) -> Result<()> {
        if let Some(metadata) = self.conversations.remove(id) {
            let conversation_file = self.storage_dir.join(format!("{}.json", id));
            if conversation_file.exists() {
                fs::remove_file(&conversation_file)
                    .context("Failed to delete conversation file")?;
            }
            self.save_index()?;
            info!("Deleted conversation: {} ({})", metadata.title, id);
        }
        Ok(())
    }
    
    /// Export conversation for sharing
    pub async fn export_conversation(&self, id: &str) -> Result<ShareableConversation> {
        let metadata = self.conversations.get(id)
            .ok_or_else(|| anyhow::anyhow!("Conversation {} not found", id))?
            .clone();
        
        let conversation = self.load_conversation(id).await?;
        
        Ok(ShareableConversation {
            metadata,
            conversation,
            export_timestamp: Utc::now(),
            aircher_version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
    
    /// Import a shareable conversation
    pub async fn import_conversation(&mut self, shareable: ShareableConversation) -> Result<String> {
        let imported_title = format!("Imported: {}", shareable.metadata.title);
        let id = self.save_conversation(
            &shareable.conversation,
            Some(imported_title),
            shareable.metadata.provider,
            shareable.metadata.model,
        ).await?;
        
        info!("Imported conversation: {}", shareable.metadata.title);
        Ok(id)
    }
    
    /// Mark conversation as shared with URL
    pub async fn share_conversation(&mut self, id: &str, share_url: String) -> Result<()> {
        let title = if let Some(metadata) = self.conversations.get_mut(id) {
            metadata.is_shared = true;
            metadata.share_url = Some(share_url.clone());
            metadata.updated_at = Utc::now();
            metadata.title.clone()
        } else {
            return Ok(());
        };
        
        self.save_index()?;
        info!("Shared conversation: {} -> {}", title, share_url);
        Ok(())
    }
    
    /// Search conversations by content or title
    pub async fn search_conversations(&self, query: &str) -> Result<Vec<&ConversationMetadata>> {
        let query = query.to_lowercase();
        let mut results = Vec::new();
        
        for metadata in self.conversations.values() {
            // Search in title
            if metadata.title.to_lowercase().contains(&query) {
                results.push(metadata);
                continue;
            }
            
            // Search in conversation content (expensive - only for small sets)
            if let Ok(conversation) = self.load_conversation(&metadata.id).await {
                for message in &conversation.messages {
                    if message.content.to_lowercase().contains(&query) {
                        results.push(metadata);
                        break;
                    }
                }
            }
        }
        
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(results)
    }
    
    /// Load conversation index from disk
    fn load_index(index_file: &PathBuf) -> Result<HashMap<String, ConversationMetadata>> {
        if !index_file.exists() {
            debug!("Conversation index does not exist, starting with empty index");
            return Ok(HashMap::new());
        }
        
        let content = fs::read_to_string(index_file)
            .context("Failed to read conversation index")?;
            
        if content.trim().is_empty() {
            debug!("Conversation index is empty, starting with empty index");
            return Ok(HashMap::new());
        }
        
        let conversations: HashMap<String, ConversationMetadata> = serde_json::from_str(&content)
            .context("Failed to parse conversation index")?;
            
        debug!("Loaded {} conversations from index", conversations.len());
        Ok(conversations)
    }
    
    /// Save conversation index to disk
    fn save_index(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.conversations)
            .context("Failed to serialize conversation index")?;
            
        fs::write(&self.index_file, content)
            .context("Failed to write conversation index")?;
            
        debug!("Saved conversation index with {} entries", self.conversations.len());
        Ok(())
    }
    
    /// Get storage statistics
    pub fn get_stats(&self) -> ConversationStats {
        let total_conversations = self.conversations.len();
        let shared_conversations = self.conversations.values()
            .filter(|m| m.is_shared)
            .count();
        let total_messages = self.conversations.values()
            .map(|m| m.message_count)
            .sum();
            
        ConversationStats {
            total_conversations,
            shared_conversations,
            total_messages,
            storage_dir: self.storage_dir.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ConversationStats {
    pub total_conversations: usize,
    pub shared_conversations: usize,
    pub total_messages: usize,
    pub storage_dir: PathBuf,
}

impl Default for ConversationStorage {
    fn default() -> Self {
        Self::new().expect("Failed to initialize conversation storage")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::conversation::{Message, MessageRole, ProjectContext, ProgrammingLanguage};
    use tempfile::TempDir;
    
    fn create_test_conversation() -> CodingConversation {
        CodingConversation {
            messages: vec![
                Message {
                    role: MessageRole::User,
                    content: "Hello, can you help me with Rust code?".to_string(),
                    tool_calls: None,
                    timestamp: Utc::now(),
                },
                Message {
                    role: MessageRole::Assistant,
                    content: "I'd be happy to help you with Rust code!".to_string(),
                    tool_calls: None,
                    timestamp: Utc::now(),
                },
            ],
            project_context: ProjectContext {
                root_path: std::env::current_dir().unwrap(),
                language: ProgrammingLanguage::Rust,
                framework: Some("cargo".to_string()),
                recent_changes: Vec::new(),
            },
            active_files: Vec::new(),
            task_list: Vec::new(),
        }
    }
    
    #[tokio::test]
    async fn test_conversation_storage() {
        let temp_dir = TempDir::new().unwrap();
        let mut storage = ConversationStorage::with_storage_dir(temp_dir.path().to_path_buf()).unwrap();
        
        let conversation = create_test_conversation();
        
        // Test saving
        let id = storage.save_conversation(
            &conversation,
            Some("Test Conversation".to_string()),
            Some("ollama".to_string()),
            Some("gpt-oss".to_string()),
        ).await.unwrap();
        
        // Test loading
        let loaded = storage.load_conversation(&id).await.unwrap();
        assert_eq!(loaded.messages.len(), conversation.messages.len());
        assert_eq!(loaded.messages[0].content, conversation.messages[0].content);
        
        // Test listing
        let conversations = storage.list_conversations();
        assert_eq!(conversations.len(), 1);
        assert_eq!(conversations[0].title, "Test Conversation");
        
        // Test export
        let exported = storage.export_conversation(&id).await.unwrap();
        assert_eq!(exported.metadata.title, "Test Conversation");
        assert_eq!(exported.conversation.messages.len(), 2);
        
        // Test deletion
        storage.delete_conversation(&id).await.unwrap();
        let conversations_after_delete = storage.list_conversations();
        assert_eq!(conversations_after_delete.len(), 0);
    }
    
    #[tokio::test]
    async fn test_search_conversations() {
        let temp_dir = TempDir::new().unwrap();
        let mut storage = ConversationStorage::with_storage_dir(temp_dir.path().to_path_buf()).unwrap();
        
        let conversation = create_test_conversation();
        let _id = storage.save_conversation(
            &conversation,
            Some("Rust Help Session".to_string()),
            None,
            None,
        ).await.unwrap();
        
        // Search by title
        let results = storage.search_conversations("rust").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Help Session");
        
        // Search by content
        let results = storage.search_conversations("happy to help").await.unwrap();
        assert_eq!(results.len(), 1);
    }
}