/// Proof of concept: TUI using UnifiedAgent through LocalClient
/// 
/// This shows how we'll refactor TuiManager to use the agent-first architecture

use anyhow::Result;
use std::sync::Arc;

// These would be imported from our crate
mod mock {
    use super::*;
    use async_trait::async_trait;
    use serde_json::Value;
    
    // Simplified versions of our real types
    pub struct UnifiedAgent;
    pub struct LocalClient {
        pub agent: Arc<UnifiedAgent>,
    }
    
    pub struct AgentResponse {
        pub content: String,
        pub session_id: String,
    }
    
    #[async_trait]
    pub trait AgentClient: Send + Sync {
        async fn create_session(&self) -> Result<String>;
        async fn send_prompt(&self, session_id: &str, message: String) -> Result<AgentResponse>;
    }
    
    #[async_trait]
    impl AgentClient for LocalClient {
        async fn create_session(&self) -> Result<String> {
            Ok("session-123".to_string())
        }
        
        async fn send_prompt(&self, session_id: &str, message: String) -> Result<AgentResponse> {
            Ok(AgentResponse {
                content: format!("Agent response to: {}", message),
                session_id: session_id.to_string(),
            })
        }
    }
}

use mock::*;

/// Refactored TuiManager structure
pub struct TuiManager {
    // OLD: agent_controller: Option<AgentController>,
    // NEW: Use client abstraction
    agent_client: Arc<dyn AgentClient>,
    session_id: String,
    messages: Vec<String>,
}

impl TuiManager {
    /// Initialize TUI with UnifiedAgent
    pub async fn new() -> Result<Self> {
        // Step 1: Create the unified agent (shared across all modes)
        let agent = Arc::new(UnifiedAgent);
        
        // Step 2: Create LocalClient for TUI (direct access, no JSON-RPC)
        let client = Arc::new(LocalClient { agent });
        
        // Step 3: Create session
        let session_id = client.create_session().await?;
        
        Ok(Self {
            agent_client: client as Arc<dyn AgentClient>,
            session_id,
            messages: Vec::new(),
        })
    }
    
    /// Send message using client abstraction
    pub async fn send_message(&mut self, message: String) -> Result<()> {
        println!("User: {}", message);
        
        // OLD WAY:
        // if let Some(ref mut controller) = self.agent_controller {
        //     let (response, _) = controller.process_message(&message, provider, model).await?;
        // }
        
        // NEW WAY: Use client abstraction
        let response = self.agent_client.send_prompt(
            &self.session_id,
            message.clone()
        ).await?;
        
        println!("Assistant: {}", response.content);
        
        // Store messages
        self.messages.push(format!("User: {}", message));
        self.messages.push(format!("Assistant: {}", response.content));
        
        Ok(())
    }
    
    /// The TUI doesn't know or care if the agent is local or remote
    pub fn is_using_local_agent(&self) -> bool {
        // This information is abstracted away
        // TUI just uses AgentClient trait
        true
    }
}

/// ACP Server would use the same agent
pub struct AcpServer {
    agent: Arc<UnifiedAgent>,
}

impl AcpServer {
    pub fn new(agent: Arc<UnifiedAgent>) -> Self {
        Self { agent }
    }
    
    pub async fn handle_json_rpc(&self, request: String) -> String {
        // Parse JSON-RPC request
        // Call agent methods
        // Return JSON-RPC response
        format!("JSON-RPC response for: {}", request)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== TUI Integration with UnifiedAgent ===\n");
    
    // Create the single agent instance
    let agent = Arc::new(UnifiedAgent);
    
    println!("1. TUI Mode (using LocalClient):");
    println!("   - Direct function calls");
    println!("   - No serialization overhead");
    println!("   - Optimal performance\n");
    
    // TUI uses LocalClient
    let mut tui = TuiManager::new().await?;
    tui.send_message("Hello from TUI".to_string()).await?;
    
    println!("\n2. ACP Mode (would use JSON-RPC):");
    println!("   - Same agent instance");
    println!("   - JSON-RPC protocol");
    println!("   - Standard compliance\n");
    
    // ACP server would use the same agent
    let acp_server = AcpServer::new(agent);
    let response = acp_server.handle_json_rpc("{\"method\": \"prompt\"}".to_string()).await;
    println!("ACP Server: {}", response);
    
    println!("\n=== Key Benefits ===");
    println!("✓ Single agent implementation");
    println!("✓ Consistent behavior across all modes");
    println!("✓ TUI maintains performance advantage");
    println!("✓ Clean separation of concerns");
    println!("✓ Easy to test (mock the client)");
    
    Ok(())
}