# Aircher Comprehensive Test Plan

## Testing Strategy

### Phase 1: Core Agent Testing
Test the UnifiedAgent directly without TUI to validate core functionality.

### Phase 2: ACP Implementation Testing  
Test the Agent Communication Protocol implementation for editor integration.

### Phase 3: TUI Integration Testing
Test the full TUI with real user interactions.

### Phase 4: Performance & Stress Testing
Test with long conversations, multiple tools, edge cases.

## Available Test Models (Ollama)

- **deepseek-r1** (5.2 GB) - Fast, good for quick tests
- **exaone-deep** (4.8 GB) - Smallest, fastest iteration
- **gpt-oss** (13 GB) - Best quality for comprehensive testing
- **gemma3n** (7.5 GB) - Mid-size alternative

## Test Coverage Requirements

### 1. Agent Core Functionality
- [ ] Message processing
- [ ] Tool execution
- [ ] Context management
- [ ] Intelligence integration
- [ ] Error handling

### 2. Tool System
- [ ] read_file
- [ ] write_file
- [ ] run_command
- [ ] search_code
- [ ] list_files
- [ ] Tool chaining

### 3. Provider Integration
- [ ] Ollama connection
- [ ] Model switching
- [ ] Streaming responses
- [ ] Error recovery
- [ ] Authentication flow

### 4. TUI Features
- [ ] Model selection (/model)
- [ ] Search command (/search)
- [ ] Conversation display
- [ ] Tool result rendering
- [ ] Collapsible sections
- [ ] TODO panel
- [ ] Keyboard shortcuts

### 5. Intelligence System
- [ ] Pattern learning
- [ ] Context enhancement
- [ ] File predictions
- [ ] Memory persistence

## Test Implementation Plan

### Step 1: Unit Tests
Create targeted tests for each component.

### Step 2: Integration Tests
Test component interactions with real Ollama.

### Step 3: End-to-End Tests
Automated TUI testing with scripted interactions.

### Step 4: Manual Validation
Human verification of complex workflows.