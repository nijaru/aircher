# Core Features Validation

## AI Chat Interface

- [ ] **Terminal UI**: Interactive chat interface renders correctly
- [ ] **Response Streaming**: Text appears incrementally as received  
- [ ] **Conversation History**: Messages persist between sessions
- [ ] **Keyboard Navigation**: Arrow keys, scrolling work properly
- [ ] **Graceful Exit**: Ctrl+C doesn't corrupt data

## LLM Provider Integration

- [ ] **OpenAI Connection**: API calls succeed with valid key
- [ ] **Claude Connection**: Anthropic API integration works
- [ ] **Provider Switching**: Can change between providers mid-conversation
- [ ] **Error Handling**: Network failures handled gracefully
- [ ] **Token Limits**: Respects model context windows

## Authentication System

- [ ] **API Key Storage**: Credentials saved securely
- [ ] **Multiple Providers**: Can store keys for different services
- [ ] **Key Validation**: Invalid keys rejected with clear messages
- [ ] **Login Command**: `aircher login` configures providers

## Configuration Management

- [ ] **Config Loading**: TOML configuration parsed correctly
- [ ] **Default Values**: Sensible defaults when config missing
- [ ] **Config Command**: `aircher config` manages settings
- [ ] **Environment Override**: Env vars take precedence

## Data Persistence

- [ ] **Conversation Storage**: Chat history saved to database
- [ ] **Session Management**: Can resume previous conversations
- [ ] **Database Integrity**: No corruption during crashes
- [ ] **Migration Support**: Schema updates work correctly

## Project Analysis

- [ ] **File Discovery**: Finds relevant project files
- [ ] **Context Building**: Builds meaningful project context
- [ ] **Gitignore Respect**: Excludes ignored files
- [ ] **Performance**: Handles large codebases efficiently

## Cross-Platform Compatibility

- [ ] **macOS**: Works on macOS 12+
- [ ] **Linux**: Functions on major distributions
- [ ] **Windows**: Basic PowerShell compatibility
- [ ] **Terminal Types**: Works with common terminal emulators

## Security Requirements

- [ ] **Credential Encryption**: API keys stored securely
- [ ] **File Permissions**: Config files have proper permissions
- [ ] **Input Validation**: User input properly sanitized
- [ ] **No Key Logging**: API keys never appear in logs