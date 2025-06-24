# MCP (Model Context Protocol) Integration Technical Specification

## Overview

The Aircher MCP Integration System provides comprehensive support for the Model Context Protocol, enabling seamless integration with external tools, resources, and services. This system manages MCP server lifecycles, handles tool execution with security controls, and provides a unified interface for extending Aircher's capabilities.

## Architecture Principles

### Extensible Tool Ecosystem
- **Server Management**: Automatic installation, configuration, and lifecycle management of MCP servers
- **Security First**: Comprehensive permission system with user confirmation workflows
- **Scoped Access**: Project-level, user-level, and system-level server configurations
- **Performance Optimization**: Efficient server communication and result caching
- **Error Resilience**: Robust error handling and server recovery mechanisms

### MCP Protocol Compliance
- **Full Protocol Support**: Complete implementation of MCP specification
- **Transport Agnostic**: Support for stdio, HTTP, and WebSocket transports
- **Versioning**: Backward compatibility and version negotiation
- **Resource Management**: Efficient handling of prompts, resources, and tools

## Core Architecture

### MCP Manager
```go
type MCPManager struct {
    // Server registry and management
    localServers    map[string]*MCPServer
    projectServers  map[string]*MCPServer
    userServers     map[string]*MCPServer
    
    // Communication
    client          *MCPClient
    serverProcesses map[string]*ServerProcess
    
    // Installation and registry
    registry        *MCPRegistry
    installer       *MCPInstaller
    
    // Security and permissions
    permissionSystem *MCPPermissionSystem
    
    // Configuration and monitoring
    config          MCPConfig
    healthMonitor   *HealthMonitor
    logger          *zerolog.Logger
    
    // Metrics and performance
    metrics         *MCPMetrics
    cache           *MCPCache
    
    mutex           sync.RWMutex
}

type MCPScope string

const (
    LocalScope   MCPScope = "local"   // System-wide servers
    ProjectScope MCPScope = "project" // Project-specific servers
    UserScope    MCPScope = "user"    // User-specific servers
)

type MCPServer struct {
    // Server identification
    Name        string            `json:"name"`
    Command     string            `json:"command"`
    Args        []string          `json:"args"`
    Env         map[string]string `json:"env"`
    
    // Configuration
    Transport   TransportType     `json:"transport"`
    Scope       MCPScope          `json:"scope"`
    Enabled     bool              `json:"enabled"`
    AutoStart   bool              `json:"auto_start"`
    
    // Capabilities
    Tools       []ToolInfo        `json:"tools"`
    Resources   []ResourceInfo    `json:"resources"`
    Prompts     []PromptInfo      `json:"prompts"`
    
    // Runtime state
    Process     *ServerProcess    `json:"-"`
    LastSeen    time.Time         `json:"last_seen"`
    Status      ServerStatus      `json:"status"`
    
    // Metadata
    Category    MCPCategory       `json:"category"`
    Version     string            `json:"version"`
    Description string            `json:"description"`
    
    // Performance metrics
    RequestCount    int64         `json:"request_count"`
    ErrorCount      int64         `json:"error_count"`
    AvgResponseTime time.Duration `json:"avg_response_time"`
}

type TransportType string

const (
    TransportStdio     TransportType = "stdio"
    TransportHTTP      TransportType = "http"
    TransportWebSocket TransportType = "websocket"
)

type ServerStatus string

const (
    StatusStopped  ServerStatus = "stopped"
    StatusStarting ServerStatus = "starting"
    StatusRunning  ServerStatus = "running"
    StatusError    ServerStatus = "error"
    StatusStopping ServerStatus = "stopping"
)

type MCPCategory string

const (
    CoreDevelopment MCPCategory = "core_development"
    WebTools        MCPCategory = "web_tools"
    Database        MCPCategory = "database"
    DevEnvironment  MCPCategory = "dev_environment"
    Knowledge       MCPCategory = "knowledge"
    Communication   MCPCategory = "communication"
    FileSystem      MCPCategory = "filesystem"
    Git             MCPCategory = "git"
    Testing         MCPCategory = "testing"
    Documentation   MCPCategory = "documentation"
)
```

### MCP Client Implementation
```go
type MCPClient struct {
    // Transport management
    transports map[string]Transport
    
    // Protocol handling
    messageHandler *MessageHandler
    responseWaiter *ResponseWaiter
    
    // Configuration
    timeout        time.Duration
    retryConfig    RetryConfig
    
    // Metrics
    requestCount   int64
    errorCount     int64
    
    logger         *zerolog.Logger
    mutex          sync.RWMutex
}

type Transport interface {
    Connect(ctx context.Context, server *MCPServer) error
    Disconnect() error
    SendMessage(ctx context.Context, message *MCPMessage) error
    ReceiveMessage(ctx context.Context) (*MCPMessage, error)
    IsConnected() bool
    Health() error
}

type MCPMessage struct {
    JSONRPC string      `json:"jsonrpc"`
    ID      interface{} `json:"id,omitempty"`
    Method  string      `json:"method,omitempty"`
    Params  interface{} `json:"params,omitempty"`
    Result  interface{} `json:"result,omitempty"`
    Error   *MCPError   `json:"error,omitempty"`
}

type MCPError struct {
    Code    int         `json:"code"`
    Message string      `json:"message"`
    Data    interface{} `json:"data,omitempty"`
}

func (mc *MCPClient) CallTool(ctx context.Context, serverName, toolName string, arguments map[string]interface{}) (*ToolResult, error) {
    server, exists := mc.getServer(serverName)
    if !exists {
        return nil, fmt.Errorf("server %s not found", serverName)
    }
    
    // Check permissions
    if !mc.permissionSystem.CheckToolPermission(toolName, arguments) {
        return nil, fmt.Errorf("permission denied for tool %s", toolName)
    }
    
    // Prepare request
    request := &MCPMessage{
        JSONRPC: "2.0",
        ID:      generateRequestID(),
        Method:  "tools/call",
        Params: map[string]interface{}{
            "name":      toolName,
            "arguments": arguments,
        },
    }
    
    // Send request with timeout
    ctx, cancel := context.WithTimeout(ctx, mc.timeout)
    defer cancel()
    
    response, err := mc.sendRequestWithRetry(ctx, server, request)
    if err != nil {
        return nil, fmt.Errorf("tool call failed: %w", err)
    }
    
    // Parse and validate response
    result, err := mc.parseToolResult(response)
    if err != nil {
        return nil, fmt.Errorf("failed to parse tool result: %w", err)
    }
    
    return result, nil
}
```

## Server Management and Installation

### MCP Registry
```go
type MCPRegistry struct {
    // Registry sources
    registryURL     string
    localRegistry   map[string]*ServerDefinition
    remoteRegistry  map[string]*ServerDefinition
    
    // Cache and updates
    cache           *RegistryCache
    updateInterval  time.Duration
    lastUpdate      time.Time
    
    // HTTP client for remote registry
    httpClient      *http.Client
    
    logger          *zerolog.Logger
}

type ServerDefinition struct {
    Name            string            `json:"name"`
    Description     string            `json:"description"`
    Category        MCPCategory       `json:"category"`
    Version         string            `json:"version"`
    
    // Installation
    InstallCommand  string            `json:"install_command"`
    InstallType     InstallType       `json:"install_type"`
    Requirements    []string          `json:"requirements"`
    
    // Runtime
    Command         string            `json:"command"`
    Args            []string          `json:"args"`
    Env             map[string]string `json:"env"`
    Transport       TransportType     `json:"transport"`
    
    // Capabilities
    Tools           []ToolDefinition  `json:"tools"`
    Resources       []ResourceDefinition `json:"resources"`
    Prompts         []PromptDefinition `json:"prompts"`
    
    // Metadata
    Homepage        string            `json:"homepage"`
    Repository      string            `json:"repository"`
    License         string            `json:"license"`
    Author          string            `json:"author"`
    
    // Dependencies
    Dependencies    []string          `json:"dependencies"`
    Conflicts       []string          `json:"conflicts"`
}

type InstallType string

const (
    InstallNPM     InstallType = "npm"
    InstallPIP     InstallType = "pip"
    InstallUVX     InstallType = "uvx"
    InstallGit     InstallType = "git"
    InstallBinary  InstallType = "binary"
    InstallDocker  InstallType = "docker"
)

func (mr *MCPRegistry) UpdateRegistry(ctx context.Context) error {
    if mr.registryURL == "" {
        return nil // No remote registry configured
    }
    
    // Fetch remote registry
    req, err := http.NewRequestWithContext(ctx, "GET", mr.registryURL, nil)
    if err != nil {
        return fmt.Errorf("failed to create request: %w", err)
    }
    
    resp, err := mr.httpClient.Do(req)
    if err != nil {
        return fmt.Errorf("failed to fetch registry: %w", err)
    }
    defer resp.Body.Close()
    
    if resp.StatusCode != http.StatusOK {
        return fmt.Errorf("registry request failed with status %d", resp.StatusCode)
    }
    
    // Parse registry response
    var registry map[string]*ServerDefinition
    if err := json.NewDecoder(resp.Body).Decode(&registry); err != nil {
        return fmt.Errorf("failed to decode registry: %w", err)
    }
    
    // Update local cache
    mr.remoteRegistry = registry
    mr.lastUpdate = time.Now()
    
    // Persist to cache
    if err := mr.cache.SaveRegistry(registry); err != nil {
        mr.logger.Warn().Err(err).Msg("Failed to save registry cache")
    }
    
    return nil
}
```

### MCP Installer
```go
type MCPInstaller struct {
    // Installation tools
    npmPath         string
    uvxPath         string
    pipPath         string
    dockerPath      string
    
    // Configuration
    cacheDir        string
    registry        *MCPRegistry
    
    // Concurrent installation management
    installQueue    chan InstallRequest
    maxConcurrent   int
    
    logger          *zerolog.Logger
}

type InstallRequest struct {
    ServerName      string
    Definition      *ServerDefinition
    Scope           MCPScope
    ResponseChannel chan InstallResult
}

type InstallResult struct {
    Success         bool
    InstalledPath   string
    Error           error
    Duration        time.Duration
}

func (mi *MCPInstaller) InstallServer(ctx context.Context, serverName string, scope MCPScope) (*InstallResult, error) {
    // Get server definition
    definition, exists := mi.registry.GetServerDefinition(serverName)
    if !exists {
        return nil, fmt.Errorf("server %s not found in registry", serverName)
    }
    
    // Check if already installed
    if mi.isServerInstalled(serverName, scope) {
        return &InstallResult{
            Success: true,
            InstalledPath: mi.getServerPath(serverName, scope),
        }, nil
    }
    
    // Perform installation based on type
    switch definition.InstallType {
    case InstallNPM:
        return mi.installNPMServer(ctx, definition, scope)
    case InstallPIP:
        return mi.installPIPServer(ctx, definition, scope)
    case InstallUVX:
        return mi.installUVXServer(ctx, definition, scope)
    case InstallGit:
        return mi.installGitServer(ctx, definition, scope)
    case InstallBinary:
        return mi.installBinaryServer(ctx, definition, scope)
    case InstallDocker:
        return mi.installDockerServer(ctx, definition, scope)
    default:
        return nil, fmt.Errorf("unsupported install type: %s", definition.InstallType)
    }
}

func (mi *MCPInstaller) installNPMServer(ctx context.Context, definition *ServerDefinition, scope MCPScope) (*InstallResult, error) {
    start := time.Now()
    
    // Prepare npm install command
    installDir := mi.getInstallDir(definition.Name, scope)
    if err := os.MkdirAll(installDir, 0755); err != nil {
        return &InstallResult{
            Success: false,
            Error:   fmt.Errorf("failed to create install directory: %w", err),
            Duration: time.Since(start),
        }, nil
    }
    
    // Execute npm install
    cmd := exec.CommandContext(ctx, mi.npmPath, "install", definition.InstallCommand)
    cmd.Dir = installDir
    cmd.Env = append(os.Environ(), "NPM_CONFIG_CACHE="+filepath.Join(mi.cacheDir, "npm"))
    
    output, err := cmd.CombinedOutput()
    if err != nil {
        return &InstallResult{
            Success: false,
            Error:   fmt.Errorf("npm install failed: %w\nOutput: %s", err, output),
            Duration: time.Since(start),
        }, nil
    }
    
    // Verify installation
    serverPath := filepath.Join(installDir, "node_modules", ".bin", definition.Command)
    if _, err := os.Stat(serverPath); err != nil {
        return &InstallResult{
            Success: false,
            Error:   fmt.Errorf("server binary not found after installation: %w", err),
            Duration: time.Since(start),
        }, nil
    }
    
    return &InstallResult{
        Success:       true,
        InstalledPath: serverPath,
        Duration:      time.Since(start),
    }, nil
}
```

## Security and Permissions System

### Permission Architecture
```go
type MCPPermissionSystem struct {
    // Permission rules
    rules           []PermissionRule
    defaultPolicy   PermissionPolicy
    
    // User interaction
    confirmations   chan PermissionRequest
    userResponses   map[string]PermissionResponse
    
    // Path controls
    allowedPaths    []string
    readOnlyPaths   []string
    deniedPaths     []string
    
    // Audit logging
    auditLogger     *AuditLogger
    
    config          PermissionConfig
    mutex           sync.RWMutex
}

type PermissionRule struct {
    ID              string              `json:"id"`
    Name            string              `json:"name"`
    Condition       PermissionCondition `json:"condition"`
    Action          PermissionAction    `json:"action"`
    Priority        int                 `json:"priority"`
    Enabled         bool                `json:"enabled"`
}

type PermissionCondition struct {
    ToolName        string   `json:"tool_name,omitempty"`
    ToolPattern     string   `json:"tool_pattern,omitempty"`
    ServerName      string   `json:"server_name,omitempty"`
    PathPattern     string   `json:"path_pattern,omitempty"`
    FileExtensions  []string `json:"file_extensions,omitempty"`
    MaxFileSize     int64    `json:"max_file_size,omitempty"`
    RequiredParams  []string `json:"required_params,omitempty"`
}

type PermissionAction string

const (
    ActionAllow            PermissionAction = "allow"
    ActionDeny             PermissionAction = "deny"
    ActionConfirm          PermissionAction = "confirm"
    ActionConfirmOnce      PermissionAction = "confirm_once"
    ActionReadOnly         PermissionAction = "read_only"
)

type ToolPermission string

const (
    FileRead        ToolPermission = "file_read"
    FileWrite       ToolPermission = "file_write"
    FileDelete      ToolPermission = "file_delete"
    FileExecute     ToolPermission = "file_execute"
    DirectoryList   ToolPermission = "directory_list"
    DirectoryCreate ToolPermission = "directory_create"
    GitRead         ToolPermission = "git_read"
    GitWrite        ToolPermission = "git_write"
    GitPush         ToolPermission = "git_push"
    DatabaseRead    ToolPermission = "database_read"
    DatabaseWrite   ToolPermission = "database_write"
    NetworkRequest  ToolPermission = "network_request"
    ProcessExecute  ToolPermission = "process_execute"
    SystemInfo      ToolPermission = "system_info"
)

func (mps *MCPPermissionSystem) CheckPermission(ctx context.Context, request *PermissionRequest) (*PermissionResponse, error) {
    mps.mutex.RLock()
    defer mps.mutex.RUnlock()
    
    // Check against rules (highest priority first)
    for _, rule := range mps.getSortedRules() {
        if !rule.Enabled {
            continue
        }
        
        if mps.matchesCondition(request, &rule.Condition) {
            response := &PermissionResponse{
                RequestID: request.ID,
                Action:    rule.Action,
                Rule:      rule.ID,
                Timestamp: time.Now(),
            }
            
            // Handle confirmation actions
            if rule.Action == ActionConfirm || rule.Action == ActionConfirmOnce {
                confirmed, err := mps.getUserConfirmation(ctx, request)
                if err != nil {
                    return nil, err
                }
                
                if confirmed {
                    response.Action = ActionAllow
                } else {
                    response.Action = ActionDeny
                }
                
                // Cache response for "confirm once" actions
                if rule.Action == ActionConfirmOnce && confirmed {
                    mps.cachePermissionResponse(request, response)
                }
            }
            
            // Audit log
            mps.auditLogger.LogPermissionCheck(request, response)
            
            return response, nil
        }
    }
    
    // No matching rule, use default policy
    response := &PermissionResponse{
        RequestID: request.ID,
        Action:    PermissionAction(mps.defaultPolicy),
        Rule:      "default",
        Timestamp: time.Now(),
    }
    
    mps.auditLogger.LogPermissionCheck(request, response)
    return response, nil
}

func (mps *MCPPermissionSystem) getUserConfirmation(ctx context.Context, request *PermissionRequest) (bool, error) {
    // Check if we have a cached response for this request pattern
    if cached := mps.getCachedResponse(request); cached != nil {
        return cached.Action == ActionAllow, nil
    }
    
    // Create confirmation dialog
    confirmation := &PermissionConfirmation{
        RequestID:   request.ID,
        ToolName:    request.ToolName,
        ServerName:  request.ServerName,
        Description: mps.generateConfirmationDescription(request),
        Arguments:   request.Arguments,
        RiskLevel:   mps.assessRiskLevel(request),
        Timeout:     30 * time.Second,
        Response:    make(chan bool, 1),
    }
    
    // Send to UI for user confirmation
    select {
    case mps.confirmations <- confirmation:
        // Wait for user response
        select {
        case confirmed := <-confirmation.Response:
            return confirmed, nil
        case <-time.After(confirmation.Timeout):
            return false, fmt.Errorf("confirmation timeout")
        case <-ctx.Done():
            return false, ctx.Err()
        }
    case <-ctx.Done():
        return false, ctx.Err()
    }
}
```

### Tool Result Processing
```go
type MCPToolResultProcessor struct {
    // Result formatters
    formatters      map[string]ResultFormatter
    
    // Content processing
    contentFilter   *ContentFilter
    sensitiveDataDetector *SensitiveDataDetector
    
    // Caching
    cache           *ResultCache
    
    // Configuration
    maxResultSize   int
    truncateOutput  bool
    
    logger          *zerolog.Logger
}

type ResultFormatter interface {
    Format(ctx context.Context, result *ToolResult) (*FormattedResult, error)
    SupportsType(resultType string) bool
}

type FormattedResult struct {
    Content         string                 `json:"content"`
    Type            string                 `json:"type"`
    Metadata        map[string]interface{} `json:"metadata"`
    Truncated       bool                   `json:"truncated"`
    SensitiveData   bool                   `json:"sensitive_data"`
    ProcessingTime  time.Duration          `json:"processing_time"`
}

func (trp *MCPToolResultProcessor) ProcessWebFetch(ctx context.Context, result *ToolResult) (*FormattedResult, error) {
    start := time.Now()
    
    // Validate result structure
    if result.Content == nil {
        return nil, fmt.Errorf("web fetch result has no content")
    }
    
    // Extract web content
    webContent, ok := result.Content.(map[string]interface{})
    if !ok {
        return nil, fmt.Errorf("invalid web fetch result format")
    }
    
    // Process different content types
    var processedContent string
    var contentType string
    
    if htmlContent, exists := webContent["html"]; exists {
        // Convert HTML to markdown for better readability
        markdown, err := trp.convertHTMLToMarkdown(htmlContent.(string))
        if err != nil {
            return nil, fmt.Errorf("failed to convert HTML to markdown: %w", err)
        }
        processedContent = markdown
        contentType = "markdown"
    } else if textContent, exists := webContent["text"]; exists {
        processedContent = textContent.(string)
        contentType = "text"
    } else {
        return nil, fmt.Errorf("no supported content type found in web fetch result")
    }
    
    // Check for sensitive data
    hasSensitiveData := trp.sensitiveDataDetector.DetectSensitiveData(processedContent)
    
    // Truncate if necessary
    truncated := false
    if len(processedContent) > trp.maxResultSize {
        processedContent = processedContent[:trp.maxResultSize] + "\n... [truncated]"
        truncated = true
    }
    
    // Filter content
    filteredContent := trp.contentFilter.FilterContent(processedContent)
    
    return &FormattedResult{
        Content:         filteredContent,
        Type:            contentType,
        Metadata:        webContent,
        Truncated:       truncated,
        SensitiveData:   hasSensitiveData,
        ProcessingTime:  time.Since(start),
    }, nil
}

func (trp *MCPToolResultProcessor) ProcessSearchResults(ctx context.Context, result *ToolResult) (*FormattedResult, error) {
    start := time.Now()
    
    // Parse search results
    searchResults, ok := result.Content.([]interface{})
    if !ok {
        return nil, fmt.Errorf("invalid search result format")
    }
    
    // Format results as markdown
    var formattedResults []string
    for i, item := range searchResults {
        if i >= 10 { // Limit to top 10 results
            break
        }
        
        resultMap, ok := item.(map[string]interface{})
        if !ok {
            continue
        }
        
        title := trp.getStringValue(resultMap, "title")
        url := trp.getStringValue(resultMap, "url")
        snippet := trp.getStringValue(resultMap, "snippet")
        
        formatted := fmt.Sprintf("## %s\n**URL:** %s\n\n%s\n", title, url, snippet)
        formattedResults = append(formattedResults, formatted)
    }
    
    content := strings.Join(formattedResults, "\n---\n\n")
    
    return &FormattedResult{
        Content:         content,
        Type:            "markdown",
        Metadata:        map[string]interface{}{"result_count": len(searchResults)},
        Truncated:       len(searchResults) > 10,
        SensitiveData:   false,
        ProcessingTime:  time.Since(start),
    }, nil
}
```

## Core MCP Servers

### Predefined Server Configurations
```go
var CoreMCPServers = map[string]*ServerDefinition{
    "filesystem": {
        Name:        "filesystem",
        Description: "File system operations and management",
        Category:    FileSystem,
        InstallType: InstallNPM,
        InstallCommand: "@modelcontextprotocol/server-filesystem",
        Command:     "mcp-server-filesystem",
        Transport:   TransportStdio,
        Tools: []ToolDefinition{
            {Name: "read_file", Description: "Read file contents"},
            {Name: "write_file", Description: "Write file contents"},
            {Name: "list_directory", Description: "List directory contents"},
            {Name: "create_directory", Description: "Create directory"},
            {Name: "delete_file", Description: "Delete file"},
            {Name: "move_file", Description: "Move or rename file"},
        },
    },
    "brave-search": {
        Name:        "brave-search",
        Description: "Web search using Brave Search API",
        Category:    WebTools,
        InstallType: InstallNPM,
        InstallCommand: "@modelcontextprotocol/server-brave-search",
        Command:     "mcp-server-brave-search",
        Transport:   TransportStdio,
        Env:         map[string]string{"BRAVE_API_KEY": "${BRAVE_API_KEY}"},
        Tools: []ToolDefinition{
            {Name: "brave_web_search", Description: "Search the web"},
        },
    },
    "git": {
        Name:        "git",
        Description: "Git repository operations",
        Category:    Git,
        InstallType: InstallNPM,
        InstallCommand: "@modelcontextprotocol/server-git",
        Command:     "mcp-server-git",
        Transport:   TransportStdio,
        Tools: []ToolDefinition{
            {Name: "git_status", Description: "Get git status"},
            {Name: "git_diff", Description: "Get git diff"},
            {Name: "git_log", Description: "Get git log"},
            {Name: "git_commit", Description: "Create git commit"},
            {Name: "git_branch", Description: "Branch operations"},
        },
    },
    "github": {
        Name:        "github",
        Description: "GitHub API integration",
        Category:    Git,
        InstallType: InstallNPM,
        InstallCommand: "@modelcontextprotocol/server-github",
        Command:     "mcp-server-github",
        Transport:   TransportStdio,
        Env:         map[string]string{"GITHUB_PERSONAL_ACCESS_TOKEN": "${GITHUB_PERSONAL_ACCESS_TOKEN}"},
        Tools: []ToolDefinition{
            {Name: "create_or_update_file", Description: "Create or update GitHub file"},
            {Name: "search_repositories", Description: "Search GitHub repositories"},
            {Name: "create_issue", Description: "Create GitHub issue"},
            {Name: "get_issue", Description: "Get GitHub issue"},
        },
    },
    "postgres": {
        Name:        "postgres",
        Description: "PostgreSQL database operations",
        Category:    Database,
        InstallType: InstallNPM,
        InstallCommand: "@modelcontextprotocol/server-postgres",
        Command:     "mcp-server-postgres",
        Transport:   TransportStdio,
        Tools: []ToolDefinition{
            {Name: "read_query", Description: "Execute SELECT query"},
            {Name: "write_query", Description: "Execute INSERT/UPDATE/DELETE query"},
            {Name: "list_tables", Description: "List database tables"},
            {Name: "describe_table", Description: "Describe table structure"},
        },
    },
    "sqlite": {
        Name:        "sqlite",
        Description: "SQLite database operations",
        Category:    Database,
        InstallType: InstallNPM,
        InstallCommand: "@modelcontextprotocol/server-sqlite",
        Command:     "mcp-server-sqlite",
        Transport:   TransportStdio,
        Tools: []ToolDefinition{
            {Name: "read_query", Description: "Execute SELECT query"},
            {Name: "write_query", Description: "Execute INSERT/UPDATE/DELETE query"},
            {Name: "list_tables", Description: "List database tables"},
            {Name: "describe_table", Description: "Describe table structure"},
        },
    },
}
```

## LLM-Database Interface Architecture

### Database Context Discovery System

The LLM needs automatic discovery of database structure and available operations. This system provides schema awareness and intelligent query generation.

```rust
pub struct DatabaseContextManager {
    // Schema discovery
    schema_cache: HashMap<String, DatabaseSchema>,
    table_relationships: HashMap<String, Vec<TableRelation>>,
    
    // MCP integration
    mcp_client: Arc<MCPClient>,
    sqlite_server_name: String,
    
    // Query optimization
    query_analyzer: QueryAnalyzer,
    performance_monitor: QueryPerformanceMonitor,
    
    // Security
    query_validator: QueryValidator,
    allowed_operations: HashSet<DatabaseOperation>,
    
    logger: Arc<Logger>,
}

#[derive(Debug, Clone)]
pub struct DatabaseSchema {
    pub database_name: String,
    pub tables: HashMap<String, TableSchema>,
    pub views: HashMap<String, ViewSchema>,
    pub indexes: Vec<IndexInfo>,
    pub last_updated: SystemTime,
}

#[derive(Debug, Clone)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub primary_keys: Vec<String>,
    pub foreign_keys: Vec<ForeignKeyInfo>,
    pub constraints: Vec<ConstraintInfo>,
    pub row_count_estimate: Option<u64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub is_unique: bool,
    pub description: Option<String>,
    pub sample_values: Vec<String>, // For better LLM understanding
}

impl DatabaseContextManager {
    pub async fn discover_schema(&mut self, database_path: &str) -> Result<DatabaseSchema> {
        // Use MCP SQLite server to discover schema
        let tables_result = self.mcp_client
            .call_tool(&self.sqlite_server_name, "list_tables", json!({
                "database": database_path
            }))
            .await?;
        
        let mut schema = DatabaseSchema {
            database_name: database_path.to_string(),
            tables: HashMap::new(),
            views: HashMap::new(),
            indexes: Vec::new(),
            last_updated: SystemTime::now(),
        };
        
        // Process each table
        if let Some(tables) = tables_result.content.as_array() {
            for table in tables {
                if let Some(table_name) = table.as_str() {
                    let table_schema = self.discover_table_schema(database_path, table_name).await?;
                    schema.tables.insert(table_name.to_string(), table_schema);
                }
            }
        }
        
        // Cache the schema
        self.schema_cache.insert(database_path.to_string(), schema.clone());
        
        Ok(schema)
    }
    
    async fn discover_table_schema(&self, database_path: &str, table_name: &str) -> Result<TableSchema> {
        // Get table structure
        let describe_result = self.mcp_client
            .call_tool(&self.sqlite_server_name, "describe_table", json!({
                "database": database_path,
                "table": table_name
            }))
            .await?;
        
        // Get sample data for better LLM understanding
        let sample_result = self.mcp_client
            .call_tool(&self.sqlite_server_name, "read_query", json!({
                "database": database_path,
                "query": format!("SELECT * FROM {} LIMIT 5", table_name)
            }))
            .await?;
        
        // Parse and build table schema
        let mut columns = Vec::new();
        if let Some(table_info) = describe_result.content.as_array() {
            for column_info in table_info {
                let column = self.parse_column_info(column_info, &sample_result)?;
                columns.push(column);
            }
        }
        
        Ok(TableSchema {
            name: table_name.to_string(),
            columns,
            primary_keys: self.extract_primary_keys(&describe_result)?,
            foreign_keys: self.extract_foreign_keys(database_path, table_name).await?,
            constraints: Vec::new(),
            row_count_estimate: self.estimate_row_count(database_path, table_name).await?,
            description: None,
        })
    }
    
    pub async fn generate_schema_context(&self) -> String {
        let mut context = String::new();
        context.push_str("# Database Schema Information\n\n");
        
        for (db_name, schema) in &self.schema_cache {
            context.push_str(&format!("## Database: {}\n\n", db_name));
            
            for (table_name, table) in &schema.tables {
                context.push_str(&format!("### Table: {}\n", table_name));
                if let Some(desc) = &table.description {
                    context.push_str(&format!("Description: {}\n", desc));
                }
                context.push_str(&format!("Estimated rows: {}\n\n", 
                    table.row_count_estimate.unwrap_or(0)));
                
                context.push_str("**Columns:**\n");
                for column in &table.columns {
                    context.push_str(&format!("- `{}` ({})", column.name, column.data_type));
                    if !column.nullable {
                        context.push_str(" NOT NULL");
                    }
                    if column.is_unique {
                        context.push_str(" UNIQUE");
                    }
                    if let Some(default) = &column.default_value {
                        context.push_str(&format!(" DEFAULT {}", default));
                    }
                    context.push('\n');
                    
                    // Add sample values for better understanding
                    if !column.sample_values.is_empty() {
                        context.push_str(&format!("  Sample values: {}\n", 
                            column.sample_values.join(", ")));
                    }
                }
                context.push('\n');
                
                // Add relationships
                if !table.foreign_keys.is_empty() {
                    context.push_str("**Foreign Keys:**\n");
                    for fk in &table.foreign_keys {
                        context.push_str(&format!("- {} → {}.{}\n", 
                            fk.column, fk.referenced_table, fk.referenced_column));
                    }
                    context.push('\n');
                }
            }
        }
        
        context
    }
}
```

### Query Generation and Validation System

```rust
pub struct QueryGenerator {
    schema_manager: Arc<DatabaseContextManager>,
    query_templates: HashMap<QueryType, Vec<QueryTemplate>>,
    safety_checker: QuerySafetyChecker,
}

#[derive(Debug)]
pub enum QueryType {
    ConversationSearch,
    KnowledgeRetrieval,
    FileContextLookup,
    SessionHistory,
    MetricsQuery,
    Custom,
}

#[derive(Debug)]
pub struct QueryTemplate {
    pub name: String,
    pub description: String,
    pub sql_template: String,
    pub parameters: Vec<QueryParameter>,
    pub safety_level: SafetyLevel,
    pub estimated_performance: PerformanceLevel,
}

impl QueryGenerator {
    pub async fn generate_query_suggestions(&self, intent: &str) -> Result<Vec<QuerySuggestion>> {
        let mut suggestions = Vec::new();
        
        // Analyze intent to determine query type
        let query_type = self.analyze_intent(intent).await?;
        
        // Get relevant templates
        let templates = self.query_templates.get(&query_type).unwrap_or(&Vec::new());
        
        for template in templates {
            if self.is_template_applicable(template, intent) {
                let suggestion = QuerySuggestion {
                    description: template.description.clone(),
                    sql: template.sql_template.clone(),
                    parameters: template.parameters.clone(),
                    safety_level: template.safety_level,
                    estimated_rows: self.estimate_result_size(&template.sql_template).await?,
                };
                suggestions.push(suggestion);
            }
        }
        
        Ok(suggestions)
    }
    
    pub async fn validate_query(&self, sql: &str) -> Result<QueryValidation> {
        let validation = QueryValidation {
            is_safe: true,
            issues: Vec::new(),
            suggestions: Vec::new(),
            estimated_performance: PerformanceLevel::Good,
        };
        
        // Check for dangerous operations
        if self.contains_dangerous_operations(sql) {
            return Ok(QueryValidation {
                is_safe: false,
                issues: vec!["Query contains potentially dangerous operations".to_string()],
                suggestions: vec!["Use read-only queries for data exploration".to_string()],
                estimated_performance: PerformanceLevel::Unknown,
            });
        }
        
        // Performance analysis
        let performance = self.analyze_query_performance(sql).await?;
        
        Ok(QueryValidation {
            is_safe: true,
            issues: Vec::new(),
            suggestions: self.generate_optimization_suggestions(sql, &performance),
            estimated_performance: performance.level,
        })
    }
}
```

### Pre-built Database Tools for LLM

```rust
pub struct AircherDatabaseTools {
    context_manager: Arc<DatabaseContextManager>,
    mcp_client: Arc<MCPClient>,
}

impl AircherDatabaseTools {
    pub async fn search_conversations(&self, query: &str, limit: Option<u32>) -> Result<Vec<ConversationResult>> {
        let sql = format!(
            "SELECT id, title, created_at, updated_at, message_count 
             FROM conversations 
             WHERE title LIKE ? OR content LIKE ? 
             ORDER BY updated_at DESC 
             LIMIT ?",
        );
        
        let result = self.mcp_client
            .call_tool("sqlite", "read_query", json!({
                "database": "conversations.db",
                "query": sql,
                "params": [
                    format!("%{}%", query),
                    format!("%{}%", query),
                    limit.unwrap_or(10)
                ]
            }))
            .await?;
        
        self.parse_conversation_results(result)
    }
    
    pub async fn get_file_context(&self, file_path: &str) -> Result<FileContext> {
        let sql = 
            "SELECT fc.*, fr.relevance_score, fr.last_accessed
             FROM file_context fc
             LEFT JOIN file_relevance fr ON fc.file_path = fr.file_path
             WHERE fc.file_path = ?";
        
        let result = self.mcp_client
            .call_tool("sqlite", "read_query", json!({
                "database": "file_index.db",
                "query": sql,
                "params": [file_path]
            }))
            .await?;
        
        self.parse_file_context(result)
    }
    
    pub async fn get_knowledge_related_to(&self, topic: &str) -> Result<Vec<KnowledgeEntry>> {
        let sql = 
            "SELECT ke.*, ks.relevance_score
             FROM knowledge_entries ke
             LEFT JOIN knowledge_search ks ON ke.id = ks.entry_id
             WHERE ke.content LIKE ? OR ke.tags LIKE ?
             ORDER BY ks.relevance_score DESC";
        
        let result = self.mcp_client
            .call_tool("sqlite", "read_query", json!({
                "database": "knowledge.db", 
                "query": sql,
                "params": [
                    format!("%{}%", topic),
                    format!("%{}%", topic)
                ]
            }))
            .await?;
        
        self.parse_knowledge_entries(result)
    }
    
    pub async fn update_file_relevance(&self, file_path: &str, relevance_score: f64) -> Result<()> {
        let sql = 
            "INSERT OR REPLACE INTO file_relevance (file_path, relevance_score, last_accessed)
             VALUES (?, ?, datetime('now'))";
        
        self.mcp_client
            .call_tool("sqlite", "write_query", json!({
                "database": "file_index.db",
                "query": sql,
                "params": [file_path, relevance_score]
            }))
            .await?;
        
        Ok(())
    }
    
    pub async fn save_conversation_insight(&self, conversation_id: &str, insight: &str, category: &str) -> Result<()> {
        let sql = 
            "INSERT INTO knowledge_entries (conversation_id, content, category, created_at)
             VALUES (?, ?, ?, datetime('now'))";
        
        self.mcp_client
            .call_tool("sqlite", "write_query", json!({
                "database": "knowledge.db",
                "query": sql,
                "params": [conversation_id, insight, category]
            }))
            .await?;
        
        Ok(())
    }
}
```

### Tool Registration and Schema Exposure

```rust
impl AircherDatabaseTools {
    pub fn register_mcp_tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "search_conversations".to_string(),
                description: "Search through conversation history".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query for conversation titles and content"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results (default: 10)",
                            "minimum": 1,
                            "maximum": 100
                        }
                    },
                    "required": ["query"]
                }),
            },
            ToolDefinition {
                name: "get_file_context".to_string(),
                description: "Get context information for a specific file".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Path to the file"
                        }
                    },
                    "required": ["file_path"]
                }),
            },
            ToolDefinition {
                name: "get_knowledge_related_to".to_string(),
                description: "Find knowledge entries related to a topic".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "topic": {
                            "type": "string",
                            "description": "Topic or keyword to search for"
                        }
                    },
                    "required": ["topic"]
                }),
            },
            ToolDefinition {
                name: "get_database_schema".to_string(),
                description: "Get database schema information for understanding data structure".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "database": {
                            "type": "string",
                            "description": "Database name (conversations, knowledge, file_index, sessions)",
                            "enum": ["conversations", "knowledge", "file_index", "sessions"]
                        }
                    },
                    "required": ["database"]
                }),
            },
            ToolDefinition {
                name: "execute_safe_query".to_string(),
                description: "Execute a validated read-only SQL query".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "database": {
                            "type": "string",
                            "description": "Target database"
                        },
                        "query": {
                            "type": "string",
                            "description": "SQL SELECT query (read-only operations only)"
                        }
                    },
                    "required": ["database", "query"]
                }),
            },
        ]
    }
    
    pub async fn generate_llm_context(&self) -> String {
        let mut context = String::new();
        
        context.push_str("# Aircher Database Interface\n\n");
        context.push_str("You have access to the following databases through MCP tools:\n\n");
        
        // Generate schema context
        context.push_str(&self.context_manager.generate_schema_context().await);
        
        // Add available tools documentation
        context.push_str("\n## Available Database Tools\n\n");
        for tool in self.register_mcp_tools() {
            context.push_str(&format!("### {}\n", tool.name));
            context.push_str(&format!("{}\n\n", tool.description));
            context.push_str("**Parameters:**\n");
            context.push_str(&format!("```json\n{}\n```\n\n", serde_json::to_string_pretty(&tool.parameters).unwrap()));
        }
        
        // Add usage examples
        context.push_str(&self.generate_usage_examples());
        
        context
    }
    
    fn generate_usage_examples(&self) -> String {
        r#"
## Usage Examples

**Search for recent conversations about a topic:**
```json
{
    "tool": "search_conversations",
    "parameters": {
        "query": "rust async",
        "limit": 5
    }
}
```

**Get context for a specific file:**
```json
{
    "tool": "get_file_context", 
    "parameters": {
        "file_path": "src/main.rs"
    }
}
```

**Find related knowledge entries:**
```json
{
    "tool": "get_knowledge_related_to",
    "parameters": {
        "topic": "error handling"
    }
}
```

**Execute custom analysis query:**
```json
{
    "tool": "execute_safe_query",
    "parameters": {
        "database": "conversations",
        "query": "SELECT COUNT(*) as total_conversations, AVG(message_count) as avg_messages FROM conversations WHERE created_at > date('now', '-7 days')"
    }
}
```
"#.to_string()
    }
}
```

## Runtime Integration and LLM Context Injection

### Startup Sequence for Database-Aware LLM

```rust
pub struct AircherLLMRuntime {
    // Core components
    llm_providers: HashMap<String, Box<dyn LLMProvider>>,
    mcp_manager: Arc<MCPManager>,
    database_tools: Arc<AircherDatabaseTools>,
    context_manager: Arc<DatabaseContextManager>,
    
    // Runtime state
    active_session: Option<SessionId>,
    conversation_context: ConversationContext,
    
    logger: Arc<Logger>,
}

impl AircherLLMRuntime {
    pub async fn initialize() -> Result<Self> {
        let mut runtime = Self::new().await?;
        
        // Step 1: Initialize MCP infrastructure
        runtime.setup_mcp_system().await?;
        
        // Step 2: Discover and register database schemas
        runtime.discover_database_schemas().await?;
        
        // Step 3: Register database tools with MCP
        runtime.register_database_tools().await?;
        
        // Step 4: Generate initial context for LLM
        runtime.prepare_llm_context().await?;
        
        Ok(runtime)
    }
    
    async fn setup_mcp_system(&mut self) -> Result<()> {
        // Install core MCP servers if not present
        let core_servers = vec!["sqlite", "filesystem", "git"];
        for server_name in core_servers {
            if !self.mcp_manager.is_server_installed(server_name).await? {
                info!("Installing MCP server: {}", server_name);
                self.mcp_manager.install_server(server_name, MCPScope::LocalScope).await?;
            }
        }
        
        // Start essential servers
        self.mcp_manager.start_server("sqlite").await?;
        self.mcp_manager.start_server("filesystem").await?;
        
        // Wait for servers to be ready
        self.mcp_manager.wait_for_servers_ready(&["sqlite", "filesystem"], Duration::from_secs(10)).await?;
        
        Ok(())
    }
    
    async fn discover_database_schemas(&mut self) -> Result<()> {
        let databases = vec![
            "conversations.db",
            "knowledge.db", 
            "file_index.db",
            "sessions.db"
        ];
        
        for db_path in databases {
            if let Ok(schema) = self.context_manager.discover_schema(db_path).await {
                info!("Discovered schema for database: {}", db_path);
                debug!("Schema: {} tables found", schema.tables.len());
            } else {
                warn!("Failed to discover schema for database: {}", db_path);
            }
        }
        
        Ok(())
    }
    
    async fn register_database_tools(&mut self) -> Result<()> {
        let tools = self.database_tools.register_mcp_tools();
        
        for tool in tools {
            self.mcp_manager.register_custom_tool(tool).await?;
            info!("Registered database tool: {}", tool.name);
        }
        
        Ok(())
    }
    
    async fn prepare_llm_context(&mut self) -> Result<()> {
        // Generate comprehensive database context
        let db_context = self.database_tools.generate_llm_context().await;
        
        // Add to conversation context
        self.conversation_context.add_system_context("database_interface", db_context);
        
        // Generate schema summaries for efficiency
        let schema_summary = self.context_manager.generate_schema_summary().await;
        self.conversation_context.add_system_context("database_schemas", schema_summary);
        
        Ok(())
    }
    
    pub async fn process_user_message(&mut self, message: &str) -> Result<String> {
        // Add current context to the conversation
        let mut messages = vec![
            Message::system(self.conversation_context.build_system_prompt()),
            Message::user(message.to_string()),
        ];
        
        // Check if this might be a database query intent
        if self.is_database_query_intent(message) {
            // Add specific database context
            let enhanced_context = self.generate_query_context(message).await?;
            messages.insert(1, Message::system(enhanced_context));
        }
        
        // Send to LLM provider with tool access
        let response = self.llm_providers
            .get("primary")
            .unwrap()
            .send_with_tools(messages, self.get_available_tools())
            .await?;
        
        // Process any tool calls in the response
        if let Some(tool_calls) = response.tool_calls {
            let tool_results = self.execute_tool_calls(tool_calls).await?;
            
            // Continue conversation with tool results
            messages.push(Message::assistant_with_tools(response.content, tool_results.clone()));
            
            let final_response = self.llm_providers
                .get("primary")
                .unwrap()
                .send(messages)
                .await?;
                
            return Ok(final_response.content);
        }
        
        Ok(response.content)
    }
    
    async fn execute_tool_calls(&self, tool_calls: Vec<ToolCall>) -> Result<Vec<ToolResult>> {
        let mut results = Vec::new();
        
        for call in tool_calls {
            let result = match call.tool_name.as_str() {
                "search_conversations" => {
                    let params: SearchParams = serde_json::from_value(call.parameters)?;
                    let conversations = self.database_tools
                        .search_conversations(&params.query, params.limit)
                        .await?;
                    ToolResult::success(serde_json::to_value(conversations)?)
                }
                "get_file_context" => {
                    let params: FileParams = serde_json::from_value(call.parameters)?;
                    let context = self.database_tools
                        .get_file_context(&params.file_path)
                        .await?;
                    ToolResult::success(serde_json::to_value(context)?)
                }
                "get_knowledge_related_to" => {
                    let params: TopicParams = serde_json::from_value(call.parameters)?;
                    let knowledge = self.database_tools
                        .get_knowledge_related_to(&params.topic)
                        .await?;
                    ToolResult::success(serde_json::to_value(knowledge)?)
                }
                "execute_safe_query" => {
                    let params: QueryParams = serde_json::from_value(call.parameters)?;
                    
                    // Validate query safety first
                    let validation = self.context_manager
                        .query_generator
                        .validate_query(&params.query)
                        .await?;
                    
                    if !validation.is_safe {
                        ToolResult::error("Query validation failed: unsafe operation detected")
                    } else {
                        // Execute through MCP SQLite server
                        let result = self.mcp_manager
                            .call_tool("sqlite", "read_query", json!({
                                "database": params.database,
                                "query": params.query
                            }))
                            .await?;
                        ToolResult::success(result.content)
                    }
                }
                _ => {
                    // Delegate to MCP system for other tools
                    self.mcp_manager.call_tool("", &call.tool_name, call.parameters).await?
                }
            };
            
            results.push(result);
        }
        
        Ok(results)
    }
    
    fn get_available_tools(&self) -> Vec<ToolDefinition> {
        let mut tools = self.database_tools.register_mcp_tools();
        
        // Add other MCP tools
        if let Ok(mcp_tools) = self.mcp_manager.list_available_tools() {
            tools.extend(mcp_tools);
        }
        
        tools
    }
    
    async fn generate_query_context(&self, message: &str) -> Result<String> {
        let mut context = String::new();
        
        // Add relevant schema information based on message content
        if message.contains("conversation") {
            context.push_str(&self.get_table_context("conversations").await);
        }
        if message.contains("file") || message.contains("code") {
            context.push_str(&self.get_table_context("file_context").await);
        }
        if message.contains("knowledge") || message.contains("learn") {
            context.push_str(&self.get_table_context("knowledge_entries").await);
        }
        
        // Add query suggestions
        if let Ok(suggestions) = self.context_manager
            .query_generator
            .generate_query_suggestions(message)
            .await
        {
            context.push_str("\n## Suggested Queries:\n");
            for suggestion in suggestions.iter().take(3) {
                context.push_str(&format!("- {}: `{}`\n", 
                    suggestion.description, suggestion.sql));
            }
        }
        
        Ok(context)
    }
}
```

### System Prompt Integration

```rust
impl DatabaseContextManager {
    pub async fn generate_system_prompt_section(&self) -> String {
        format!(r#"
# Database Access

You have access to Aircher's multi-database system through specialized tools:

## Available Databases
- **conversations.db**: Chat history, user interactions, conversation metadata
- **knowledge.db**: Accumulated insights, learned patterns, topic associations  
- **file_index.db**: File context, relevance scores, project structure analysis
- **sessions.db**: User sessions, preferences, workspace state

## Key Capabilities
1. **Search Conversations**: Find past discussions, topics, solutions
2. **Retrieve Knowledge**: Access accumulated insights and patterns
3. **File Context**: Understand project structure and file relationships
4. **Custom Queries**: Execute safe, read-only SQL for specific analysis

## Database Schema Summary
{}

## Usage Guidelines
- Use `search_conversations` for finding past discussions
- Use `get_knowledge_related_to` for topic-based insights
- Use `get_file_context` for understanding specific files
- Use `execute_safe_query` for custom analysis (read-only only)
- Always validate query safety before execution
- Prefer specific tools over raw SQL when possible

The database interface provides full context awareness for intelligent assistance.
"#, self.generate_schema_context().await)
    }
}
```

var RecommendedMCPServers = map[string]*ServerDefinition{
    "puppeteer": {
        Name:        "puppeteer",
        Description: "Web scraping and browser automation",
        Category:    WebTools,
        InstallType: InstallNPM,
        InstallCommand: "@modelcontextprotocol/server-puppeteer",
        Command:     "mcp-server-puppeteer",
        Transport:   TransportStdio,
        Tools: []ToolDefinition{
            {Name: "puppeteer_screenshot", Description: "Take website screenshot"},
            {Name: "puppeteer_click", Description: "Click element on page"},
            {Name: "puppeteer_fill", Description: "Fill form fields"},
            {Name: "puppeteer_extract", Description: "Extract data from page"},
        },
    },
    "memory": {
        Name:        "memory",
        Description: "Persistent memory and knowledge management",
        Category:    Knowledge,
        InstallType: InstallNPM,
        InstallCommand: "@modelcontextprotocol/server-memory",
        Command:     "mcp-server-memory",
        Transport:   TransportStdio,
        Tools: []ToolDefinition{
            {Name: "create_memory", Description: "Create new memory"},
            {Name: "search_memory", Description: "Search memories"},
            {Name: "update_memory", Description: "Update existing memory"},
            {Name: "delete_memory", Description: "Delete memory"},
        },
    },
    "fetch": {
        Name:        "fetch",
        Description: "HTTP requests and web content fetching",
        Category:    WebTools,
        InstallType: InstallNPM,
        InstallCommand: "@modelcontextprotocol/server-fetch",
        Command:     "mcp-server-fetch",
        Transport:   TransportStdio,
        Tools: []ToolDefinition{
            {Name: "fetch", Description: "Fetch web content"},
        },
    },