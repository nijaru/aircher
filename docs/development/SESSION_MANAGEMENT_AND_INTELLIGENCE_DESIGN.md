# Session Management and Intelligence Engine Design

## Overview

This document outlines the design decisions for Aircher's session management and intelligence engine, evolved through extensive analysis of user needs, LLM agent requirements, and implementation complexity trade-offs.

## Core Requirements

### User Experience Goals
- **Invisible by default**: Session management should "just work" without user management
- **TUI-focused**: Primary usage is `aircher` (TUI), with `aircher "message"` as one-shot CLI
- **Project-aware**: Sessions tied to working directory/project context
- **Effective AI assistance**: LLM should understand codebase and provide accurate help
- **Cost-efficient**: Balance intelligence gathering costs with improved effectiveness

### Technical Requirements
- **Fast startup**: TUI should start quickly with relevant context
- **Persistent memory**: Remember project state between sessions
- **Local-first**: No team coordination complexity
- **Reliable**: Graceful degradation when intelligence systems fail

## Final Design Decisions

### Session Management Model
- **One session per project directory**: Simple, predictable behavior
- **Auto-session creation**: Every TUI instance creates/continues project session
- **Project detection**: Look for `.aircher/` directory, walk up tree like git
- **Auto-create files**: Create `.aircher/` directory and necessary files on first use

### Intelligence Engine Architecture

#### Core Structure
```rust
struct IntelligenceEngine {
    codebase: CodebaseKnowledge,      // File purposes, dependencies, structure
    project: ProjectKnowledge,        // Architecture, decisions, tech stack
    conversations: ConversationMemory, // Session history, insights, corrections
    patterns: LearnedPatterns,        // Work preferences, common tasks
}
```

#### Knowledge Components
- **CodebaseKnowledge**: File purposes, dependencies, key functions
- **ProjectKnowledge**: Architectural decisions, tech stack, current focus
- **ConversationMemory**: Session history, extracted insights, user corrections
- **LearnedPatterns**: Work preferences, common task patterns, error solutions

### Context Strategy: Hybrid Approach

#### 1. Rich Context Injection (System Prompt)
- Pre-computed project context injected into every conversation
- Includes: recent changes, architecture overview, current focus, key decisions
- Updated automatically in background, ready immediately

#### 2. Smart Tools for Deep Dives
```rust
#[tool] fn understand_file(path: &str) -> String;
#[tool] fn search_project_knowledge(query: &str) -> String;
```

#### 3. Background Intelligence Gathering
- **File change analysis**: Triggered on significant changes (debounced)
- **Conversation processing**: Extract insights after natural conversation boundaries
- **Git integration**: Learn from commit messages and diffs

## Options Considered and Rejected

### Session Management Alternatives

#### Complex Session Management (Rejected)
- **Considered**: Multiple sessions per project, session switching, complex CLI commands
- **Rejected because**: Added complexity without clear user benefit, broke TUI flow

#### Team-Wide Sessions (Rejected)
- **Considered**: Shared session intelligence across team members
- **Rejected because**: Social/political complexity, privacy concerns, maintenance nightmare

#### Magic Session Detection (Rejected)
- **Considered**: Auto-detect when to start new vs continue sessions based on content
- **Rejected because**: Unreliable intent detection, user prefers explicit control

### Intelligence Engine Alternatives

#### Intent-Driven Context Loading (Rejected)
- **Considered**: Analyze user intent first, load specific context types
- **Rejected because**: Intent classification unreliable, added latency, over-engineering

#### Comprehensive Database Structure (Rejected)
- **Considered**: Separate specialized databases for different context types
- **Rejected because**: LLM orchestration complexity, consistency problems

#### Real-Time LLM Analysis (Rejected)
- **Considered**: LLM analyzes every file change immediately
- **Rejected because**: Too expensive, breaks development flow

#### Tools-Heavy Approach (Rejected)
- **Considered**: Many specialized tools for different query types
- **Rejected because**: Tool call latency, LLM decision fatigue

## Key Design Principles

### 1. Local-Only Intelligence
- No team coordination needed
- Privacy by default
- Works for any team size/structure
- Natural learning through code and conversation

### 2. Conversation-First Learning
- AI learns through natural Q&A rather than complex analysis
- User can correct misunderstandings easily
- Context builds organically through usage

### 3. Background + Foreground Balance
- Heavy analysis happens in background (file changes, git commits)
- Conversation starts fast with pre-computed context
- Deep dives available on-demand via tools

### 4. Graceful Degradation
- Basic functionality works without intelligence system
- Fallback to simple file listing + git status
- No hard dependencies on background processing

## Implementation Priorities

### Phase 1: Basic Session Management
1. Project detection (`.aircher/` directory walking)
2. Simple session persistence (conversation history)
3. Basic file change monitoring
4. TUI integration

### Phase 2: Core Intelligence
1. Background file analysis
2. Conversation insight extraction
3. Rich context injection
4. Basic understanding tools

### Phase 3: Advanced Features
1. Pattern learning
2. Proactive suggestions
3. Context compaction
4. Performance optimization

## Open Questions for Evaluation

1. **Context window management**: How to handle projects with massive context over time?
2. **Analysis triggers**: Optimal timing for background intelligence gathering?
3. **Quality validation**: How to ensure intelligence accuracy and handle corrections?
4. **Resource usage**: CPU/memory impact of file watching and analysis?
5. **Migration path**: How to evolve intelligence structures over time?

## Expected Benefits

- **Reduced repetition**: AI won't ask same questions about project structure
- **Better accuracy**: Contextual understanding reduces mistakes
- **Faster development**: AI can suggest relevant files/patterns immediately
- **Learning over time**: System gets more helpful with continued use
- **Cost efficiency**: Upfront intelligence investment reduces token usage long-term