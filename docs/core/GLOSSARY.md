# Aircher Glossary

## Core Concepts

**Aircher**  
AI-powered terminal assistant that provides intelligent command-line interfaces with multi-LLM provider support.

**Agent**  
An AI assistant instance that can execute commands, analyze code, and provide development support through the terminal interface.

**Context Management**  
System for intelligently selecting and maintaining relevant information across conversations, including file relevance scoring and hierarchical context organization.

**Provider Interface**  
Abstraction layer that enables support for multiple LLM providers (OpenAI, Claude, Gemini, Ollama) through a unified API.

## Architecture Terms

**Clean Architecture**  
Design pattern organizing code into layers with clear boundaries, ensuring domain logic independence from external concerns.

**Multi-Database Pattern**  
Architectural approach using separate SQLite databases for different data concerns (conversations, knowledge, file_index, sessions).

**Trait-Based Design**  
Rust pattern using traits to define shared behavior across types, enabling flexible and extensible code organization.

**MCP (Model Context Protocol)**  
Protocol for extending AI capabilities through structured tool integration and context sharing.

## Technical Components

**TUI (Terminal User Interface)**  
Text-based user interface framework (Ratatui) providing rich terminal interactions without requiring a graphical environment.

**Streaming Interface**  
Real-time data processing system that handles continuous LLM response streams for responsive user interactions.

**File Index**  
Database-backed system for tracking and scoring file relevance to improve context selection efficiency.

**Session Management**  
System for maintaining conversation state, user preferences, and work context across multiple interactions.

## Development Terms

**Provider Pattern**  
Design pattern implementing a unified interface for different service implementations (LLM providers).

**Async-First**  
Development approach prioritizing asynchronous operations using Rust's tokio runtime for non-blocking execution.

**SQLx**  
Rust SQL toolkit providing compile-time checked queries and async database operations.

**Tracing**  
Structured logging framework for debugging and monitoring application behavior.

## LLM Integration

**Streaming Response**  
Real-time token-by-token delivery of LLM outputs, enabling responsive user experience during generation.

**Token Optimization**  
Strategies for minimizing context size while maintaining effectiveness, crucial for cost and performance.

**Provider Abstraction**  
Unified interface allowing seamless switching between different LLM services without code changes.

**Context Hierarchy**  
Organized structure for managing different types of context (global, session, conversation, task-specific).

## Data Management

**Conversation Store**  
Database containing chat history, user interactions, and response metadata for continuity.

**Knowledge Base**  
Structured storage for accumulated insights, patterns, and learned information across sessions.

**File Relevance Scoring**  
Algorithm determining which files are most pertinent to current tasks or conversations.

**Session Persistence**  
Maintaining user state, preferences, and work context between application restarts.

## Configuration

**TOML Configuration**  
Human-readable configuration format used for application settings, credentials, and provider configurations.

**Credential Management**  
Secure storage and handling of API keys, tokens, and authentication information.

**Provider Configuration**  
Settings specific to each LLM provider including endpoints, models, and behavioral parameters.

## Quality Assurance

**Clippy**  
Rust linting tool providing code quality suggestions and catching common mistakes.

**Cargo Test**  
Rust's built-in testing framework for unit tests, integration tests, and documentation tests.

**Validation Gates**  
Automated checks ensuring code quality, functionality, and architectural compliance before commits.

## Workflow Terms

**Sprint Tasks**  
Short-term development objectives organized in priority sequences (e.g., SPRINT-001, SPRINT-002).

**Task Status Progression**  
Workflow states: `pending` → `in_progress` → `completed` → archived to `completed.json`.

**Autonomous Work**  
AI agent capability to execute complete tasks with minimal human intervention using structured task definitions.

**Context Loading**  
Process of selecting and loading only task-relevant documentation to optimize token usage.

## Error Handling

**Error Recovery**  
Systematic approach to diagnosing and resolving issues using structured troubleshooting documentation.

**Diagnostic Tools**  
Built-in capabilities for system health checks, configuration validation, and performance analysis.

**Failure Modes**  
Common error patterns and their resolution strategies documented for rapid problem solving.