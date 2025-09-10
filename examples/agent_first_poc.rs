// Proof of concept: Agent-first architecture
// Shows how TUI and ACP modes can share the same agent implementation

use async_trait::async_trait;
use serde_json::Value;

// Core agent that is ACP-native
pub struct UnifiedAgent {
    // Agent state (tools, intelligence, context, etc.)
}

// ACP Agent trait (simplified for POC)
#[async_trait]
trait Agent: Send + Sync {
    async fn initialize(&self) -> Result<Value, String>;
    async fn prompt(&self, message: String) -> Result<String, String>;
}

#[async_trait]
impl Agent for UnifiedAgent {
    async fn initialize(&self) -> Result<Value, String> {
        Ok(serde_json::json!({
            "capabilities": ["tools", "streaming"],
            "models": ["gpt-4", "claude-3.5"]
        }))
    }
    
    async fn prompt(&self, message: String) -> Result<String, String> {
        // This is where agent logic happens ONCE
        // Tools, intelligence, context - all here
        Ok(format!("Agent received: {}", message))
    }
}

// Client abstraction - how different frontends access the agent
#[async_trait]
trait AgentClient: Send + Sync {
    async fn send_message(&self, message: String) -> Result<String, String>;
}

// Local client for TUI - direct function calls
struct LocalClient {
    agent: std::sync::Arc<UnifiedAgent>,
}

#[async_trait]
impl AgentClient for LocalClient {
    async fn send_message(&self, message: String) -> Result<String, String> {
        // Direct call - no JSON-RPC overhead
        self.agent.prompt(message).await
    }
}

// Remote client for ACP - JSON-RPC over stdio
struct RemoteClient {
    // Would contain JSON-RPC connection
}

#[async_trait]
impl AgentClient for RemoteClient {
    async fn send_message(&self, message: String) -> Result<String, String> {
        // Serialize to JSON-RPC, send over stdio, parse response
        Ok(format!("Remote: {}", message))
    }
}

// TUI Application - uses AgentClient, not aware of implementation
struct TuiApp {
    client: Box<dyn AgentClient>,
}

impl TuiApp {
    async fn handle_user_input(&mut self, input: String) {
        // TUI doesn't know if agent is local or remote
        match self.client.send_message(input).await {
            Ok(response) => println!("Response: {}", response),
            Err(e) => println!("Error: {}", e),
        }
    }
}

#[tokio::main]
async fn main() {
    let agent = std::sync::Arc::new(UnifiedAgent {});
    
    // TUI mode - uses local client
    let local_client = LocalClient { 
        agent: agent.clone() 
    };
    let mut tui = TuiApp { 
        client: Box::new(local_client) 
    };
    tui.handle_user_input("Hello from TUI".to_string()).await;
    
    // ACP mode would use the same agent
    // let server = AcpServer::new(agent);
    // server.run_stdio().await;
    
    println!("\nKey insight: Same agent, different access methods!");
    println!("- TUI gets direct access (fast, no serialization)");
    println!("- ACP gets JSON-RPC access (standard protocol)");
    println!("- Agent logic written once, tested once, works everywhere");
}