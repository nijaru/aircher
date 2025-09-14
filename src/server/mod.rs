//! ACP Server implementation for editor integration
//! 
//! Provides JSON-RPC over stdio for Zed, VS Code, and other ACP-compatible editors

#[cfg(feature = "acp")]
pub mod stdio;

#[cfg(feature = "acp")]
pub use stdio::*;

#[cfg(not(feature = "acp"))]
pub struct AcpServer;

#[cfg(not(feature = "acp"))]
impl AcpServer {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn run_stdio(self) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("ACP support not enabled. Compile with --features acp"))
    }
}