use anyhow::Result;
use clap::{Args, Subcommand};

#[cfg(feature = "mcp")]
use crate::mcp::{
    McpConfig, McpClientManager,
    McpServerConfig, McpServerType,
};

#[derive(Debug, Args)]
pub struct McpArgs {
    #[command(subcommand)]
    pub command: McpCommand,
}

#[derive(Debug, Subcommand)]
pub enum McpCommand {
    /// List configured MCP servers
    List {
        /// Show detailed server information
        #[arg(long)]
        verbose: bool,
    },
    /// Add a new MCP server configuration
    Add {
        /// Server name (unique identifier)
        name: String,
        /// Server type
        #[arg(value_enum)]
        server_type: ServerType,
        /// Command to execute (for stdio servers)
        #[arg(long, required_if_eq("server_type", "stdio"))]
        command: Option<String>,
        /// Command arguments (for stdio servers)
        #[arg(long, value_delimiter = ' ')]
        args: Option<Vec<String>>,
        /// Server URL (for HTTP servers)
        #[arg(long, required_if_eq("server_type", "http"))]
        url: Option<String>,
        /// API key for authentication
        #[arg(long)]
        api_key: Option<String>,
        /// Server description
        #[arg(short, long)]
        description: Option<String>,
        /// Tags for categorization
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
    },
    /// Remove an MCP server configuration
    Remove {
        /// Server name to remove
        name: String,
        /// Force removal without confirmation
        #[arg(long)]
        force: bool,
    },
    /// Connect to an MCP server
    Connect {
        /// Server name to connect to
        name: String,
        /// Connection timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
    },
    /// Disconnect from an MCP server
    Disconnect {
        /// Server name to disconnect from
        name: String,
    },
    /// Show server status
    Status {
        /// Server name (show all if not specified)
        name: Option<String>,
    },
    /// List available tools from connected servers
    Tools {
        /// Filter by server name
        #[arg(long)]
        server: Option<String>,
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
    },
    /// List available resources from connected servers
    Resources {
        /// Filter by server name
        #[arg(long)]
        server: Option<String>,
    },
    /// Execute a tool from an MCP server
    Call {
        /// Tool name (format: server_name.tool_name or just tool_name)
        tool: String,
        /// Tool arguments as JSON
        #[arg(long)]
        args: Option<String>,
        /// Pretty print output
        #[arg(long)]
        pretty: bool,
    },
    /// Get a resource from an MCP server
    Get {
        /// Resource URI
        uri: String,
        /// Server name (auto-detect if not specified)
        #[arg(long)]
        server: Option<String>,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ServerType {
    Stdio,
    Http,
}

#[cfg(feature = "mcp")]
async fn load_or_create_config() -> Result<McpConfig> {
    match McpConfig::load().await {
        Ok(config) => Ok(config),
        Err(_) => {
            // Create default config if none exists
            let config = McpConfig::default();
            config.save().await?;
            Ok(config)
        }
    }
}

#[cfg(feature = "mcp")]
pub async fn handle_mcp_command(args: McpArgs) -> Result<()> {
    match args.command {
        McpCommand::List { verbose } => {
            let config = load_or_create_config().await?;
            
            if config.servers.is_empty() {
                println!("No MCP servers configured.");
                println!("Add a server with: aircher mcp add <name> <type> [options]");
                return Ok(());
            }
            
            println!("Configured MCP servers:\n");
            for (name, server) in &config.servers {
                if verbose {
                    println!("🔌 {}", name);
                    println!("   Type: {:?}", server.server_type);
                    if let Some(desc) = &server.description {
                        println!("   Description: {}", desc);
                    }
                    if !server.tags.is_empty() {
                        println!("   Tags: {}", server.tags.join(", "));
                    }
                    match &server.server_type {
                        McpServerType::Local { command, args, .. } => {
                            println!("   Command: {} {}", command, args.join(" "));
                        }
                        McpServerType::Remote { url, auth_type, .. } => {
                            println!("   URL: {}", url);
                            match auth_type {
                                crate::mcp::AuthType::None => {},
                                crate::mcp::AuthType::ApiKey { .. } => {
                                    println!("   Authentication: API Key configured");
                                }
                                crate::mcp::AuthType::Bearer { .. } => {
                                    println!("   Authentication: Bearer token configured");
                                }
                                crate::mcp::AuthType::OAuth { .. } => {
                                    println!("   Authentication: OAuth configured");
                                }
                            }
                        }
                        McpServerType::Docker { image, args, .. } => {
                            println!("   Docker image: {} {}", image, args.join(" "));
                        }
                    }
                    println!();
                } else {
                    let type_str = match &server.server_type {
                        McpServerType::Local { .. } => "local",
                        McpServerType::Remote { .. } => "remote",
                        McpServerType::Docker { .. } => "docker",
                    };
                    let desc = server.description.as_deref().unwrap_or("No description");
                    println!("  {} ({}) - {}", name, type_str, desc);
                }
            }
            
            if !verbose {
                println!("\nUse --verbose for detailed information");
            }
        }
        
        McpCommand::Add {
            name,
            server_type,
            command,
            args,
            url,
            api_key,
            description,
            tags,
        } => {
            let mut config = load_or_create_config().await?;
            
            if config.servers.contains_key(&name) {
                return Err(anyhow::anyhow!("Server '{}' already exists", name));
            }
            
            let server_config = match server_type {
                ServerType::Stdio => {
                    let command = command.ok_or_else(|| anyhow::anyhow!("Command required for stdio server"))?;
                    McpServerConfig {
                        name: name.clone(),
                        enabled: true,
                        server_type: McpServerType::Local {
                            command,
                            args: args.unwrap_or_default(),
                            working_directory: None,
                            env: std::collections::HashMap::new(),
                        },
                        timeout_seconds: Some(30),
                        auto_reconnect: true,
                        description,
                        tags: tags.unwrap_or_default(),
                    }
                }
                ServerType::Http => {
                    let url = url.ok_or_else(|| anyhow::anyhow!("URL required for HTTP server"))?;
                    McpServerConfig {
                        name: name.clone(),
                        enabled: true,
                        server_type: McpServerType::Remote {
                            url,
                            auth_type: if let Some(key) = api_key {
                                crate::mcp::AuthType::ApiKey {
                                    header: "X-API-Key".to_string(),
                                    value: key,
                                }
                            } else {
                                crate::mcp::AuthType::None
                            },
                            headers: std::collections::HashMap::new(),
                        },
                        timeout_seconds: Some(30),
                        auto_reconnect: true,
                        description,
                        tags: tags.unwrap_or_default(),
                    }
                }
            };
            
            config.servers.insert(name.clone(), server_config);
            config.save().await?;
            
            println!("✅ Added MCP server '{}'", name);
            println!("Connect with: aircher mcp connect {}", name);
        }
        
        McpCommand::Remove { name, force } => {
            let mut config = load_or_create_config().await?;
            
            if !config.servers.contains_key(&name) {
                return Err(anyhow::anyhow!("Server '{}' not found", name));
            }
            
            if !force {
                println!("Remove server '{}'? This cannot be undone. [y/N]", name);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
            
            config.servers.remove(&name);
            config.save().await?;
            
            println!("✅ Removed MCP server '{}'", name);
        }
        
        McpCommand::Connect { name, timeout } => {
            let config = load_or_create_config().await?;
            let mut manager = McpClientManager::new(config).await?;
            
            println!("🔌 Connecting to '{}'...", name);
            
            match tokio::time::timeout(
                std::time::Duration::from_secs(timeout),
                manager.connect_server(&name)
            ).await {
                Ok(Ok(())) => {
                    println!("✅ Connected to '{}'", name);
                    
                    // Show available tools and resources
                    if let Ok(tools) = manager.list_tools(&name).await {
                        if let Some(tools) = tools {
                            println!("\nAvailable tools: {}", tools.len());
                            for tool in tools.iter().take(5) {
                                println!("  - {}", tool.name);
                            }
                            if tools.len() > 5 {
                                println!("  ... and {} more", tools.len() - 5);
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    println!("❌ Failed to connect: {}", e);
                }
                Err(_) => {
                    println!("❌ Connection timeout after {} seconds", timeout);
                }
            }
        }
        
        McpCommand::Disconnect { name } => {
            let config = load_or_create_config().await?;
            let mut manager = McpClientManager::new(config).await?;
            
            manager.disconnect_server(&name).await?;
            println!("✅ Disconnected from '{}'", name);
        }
        
        McpCommand::Status { name } => {
            let config = load_or_create_config().await?;
            let server_names: Vec<String> = config.servers.keys().cloned().collect();
            let manager = McpClientManager::new(config).await?;
            
            if let Some(server_name) = name {
                // Show status for specific server
                let status = manager.get_connection_status(&server_name).await;
                match status {
                    Some(status) => {
                        println!("Server '{}': {:?}", server_name, status);
                    }
                    None => {
                        println!("Server '{}' not found", server_name);
                    }
                }
            } else {
                // Show status for all servers
                println!("MCP Server Status:\n");
                for server_name in &server_names {
                    let status = manager.get_connection_status(server_name).await;
                    let status_str = match status {
                        Some(s) => format!("{:?}", s),
                        None => "Not configured".to_string(),
                    };
                    println!("  {} - {}", server_name, status_str);
                }
            }
        }
        
        McpCommand::Tools { server, tag } => {
            let config = load_or_create_config().await?;
            let manager = McpClientManager::new(config).await?;
            
            println!("Available MCP tools:\n");
            
            let servers = if let Some(server_name) = server {
                vec![server_name]
            } else {
                manager.list_connected_servers().await
            };
            
            let mut total_tools = 0;
            for server_name in servers {
                if let Ok(Some(tools)) = manager.list_tools(&server_name).await {
                    // Filter by tag if specified
                    let tools: Vec<_> = if let Some(tag) = &tag {
                        tools.into_iter()
                            .filter(|t| t.description.as_ref().map_or(false, |d| d.contains(tag)))
                            .collect()
                    } else {
                        tools
                    };
                    
                    if !tools.is_empty() {
                        println!("📦 {} ({} tools):", server_name, tools.len());
                        for tool in &tools {
                            println!("  - {}: {}", tool.name, tool.description.as_deref().unwrap_or("No description"));
                        }
                        println!();
                        total_tools += tools.len();
                    }
                }
            }
            
            if total_tools == 0 {
                println!("No tools found. Make sure servers are connected.");
                println!("Connect with: aircher mcp connect <server>");
            } else {
                println!("Total: {} tools available", total_tools);
            }
        }
        
        McpCommand::Resources { server } => {
            let config = load_or_create_config().await?;
            let manager = McpClientManager::new(config).await?;
            
            println!("Available MCP resources:\n");
            
            let servers = if let Some(server_name) = server {
                vec![server_name]
            } else {
                manager.list_connected_servers().await
            };
            
            let mut total_resources = 0;
            for server_name in servers {
                if let Ok(Some(resources)) = manager.list_resources(&server_name).await {
                    if !resources.is_empty() {
                        println!("📚 {} ({} resources):", server_name, resources.len());
                        for resource in &resources {
                            println!("  - {}: {}", resource.uri, resource.description.as_deref().unwrap_or("No description"));
                        }
                        println!();
                        total_resources += resources.len();
                    }
                }
            }
            
            if total_resources == 0 {
                println!("No resources found. Make sure servers are connected.");
            }
        }
        
        McpCommand::Call { tool, args, pretty } => {
            let config = load_or_create_config().await?;
            let manager = McpClientManager::new(config).await?;
            
            // Parse tool name (could be "server.tool" or just "tool")
            let (server_name, tool_name) = if tool.contains('.') {
                let parts: Vec<&str> = tool.splitn(2, '.').collect();
                (Some(parts[0].to_string()), parts[1].to_string())
            } else {
                (None, tool)
            };
            
            // Parse arguments
            let args_value = if let Some(args_str) = args {
                serde_json::from_str(&args_str)?
            } else {
                serde_json::Value::Object(serde_json::Map::new())
            };
            
            println!("🔧 Calling tool '{}'...", tool_name);
            
            // Find and call the tool
            let result = if let Some(server) = server_name {
                manager.call_tool(&server, &tool_name, args_value).await
            } else {
                // Try to find tool in any connected server
                let servers = manager.list_connected_servers().await;
                let mut found = None;
                
                for server in servers {
                    if let Ok(Some(tools)) = manager.list_tools(&server).await {
                        if tools.iter().any(|t| t.name == tool_name) {
                            found = Some(server);
                            break;
                        }
                    }
                }
                
                match found {
                    Some(server) => manager.call_tool(&server, &tool_name, args_value).await,
                    None => Err(anyhow::anyhow!("Tool '{}' not found in any connected server", tool_name)),
                }
            };
            
            match result {
                Ok(mcp_result) => {
                    match mcp_result.result {
                        Ok(response) => {
                            println!("✅ Tool executed successfully\n");
                            if pretty {
                                println!("{}", serde_json::to_string_pretty(&response)?);
                            } else {
                                println!("{}", response);
                            }
                        }
                        Err(e) => {
                            println!("❌ Tool execution failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to call tool: {}", e);
                }
            }
        }
        
        McpCommand::Get { uri, server } => {
            let config = load_or_create_config().await?;
            let manager = McpClientManager::new(config).await?;
            
            println!("📚 Getting resource '{}'...", uri);
            
            let result = if let Some(server_name) = server {
                manager.get_resource(&server_name, &uri).await
            } else {
                // Try to find resource in any connected server
                let servers = manager.list_connected_servers().await;
                let mut found = None;
                
                for server in servers {
                    if let Ok(Some(resources)) = manager.list_resources(&server).await {
                        if resources.iter().any(|r| r.uri == uri) {
                            found = Some(server);
                            break;
                        }
                    }
                }
                
                match found {
                    Some(server) => manager.get_resource(&server, &uri).await,
                    None => Err(anyhow::anyhow!("Resource '{}' not found in any connected server", uri)),
                }
            };
            
            match result {
                Ok(mcp_result) => {
                    match mcp_result.result {
                        Ok(content) => {
                            println!("✅ Resource retrieved\n");
                            println!("{:#?}", content);
                        }
                        Err(e) => {
                            println!("❌ Failed to get resource: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to access resource: {}", e);
                }
            }
        }
    }
    
    Ok(())
}

#[cfg(not(feature = "mcp"))]
pub async fn handle_mcp_command(_args: McpArgs) -> Result<()> {
    println!("MCP support is not enabled in this build.");
    println!("Rebuild with: cargo build --features mcp");
    Ok(())
}