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