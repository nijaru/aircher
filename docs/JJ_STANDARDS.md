# JJ + AI Agent Standards & Workflow Guide
*Complete Guide for Jujutsu Version Control with AI Agents*

## Why JJ Works Great with AI Agents ✅

**JJ (Jujutsu)** is a Git-compatible version control system that **automatically captures all changes** without manual commits. This makes it perfect for AI agents because:

1. **No commit anxiety** - Everything is auto-saved, agents can't lose work
2. **Clean up later** - Messy AI work can be squashed into logical commits
3. **Always recoverable** - `jj op undo` can fix any agent mistakes
4. **Git compatible** - Works with existing Git repos and GitHub

## Quick Start for AI Agents

1. **Initialize**: `jj git init --colocate` in existing repo OR `jj init` for new
2. **Before AI Work**: `jj new` to create sandbox revision 
3. **Core Rule**: Let jj auto-track, clean up with `jj squash`/`jj split` afterward
4. **Safety Net**: Everything is undoable with `jj op undo`/`jj op restore`
5. **Clean History**: Use `jj squash` before `jj git push`

## Priority Levels
- **MUST** = Always required, no exceptions
- **SHOULD** = Default behavior unless justified  
- **MAY** = Optional based on context

## Core Principles (jj + AI Workflow)

1. **Automatic Capture** - jj records all changes instantly, no manual commits
2. **Fearless Experimentation** - Every operation is undoable via operation log
3. **Sandbox Revisions** - Create throwaway commits for AI experiments
4. **Clean History** - Squash/split messy AI work into logical commits
5. **Always Recoverable** - `jj op log` shows everything, `jj op undo` fixes mistakes

## Initial Configuration (IMPORTANT)

### Fix Terminal Escape Sequences Issue
**Problem**: jj may output terminal control codes like `[?1049h[?1000h` to your text entry
**Solution**: Configure jj to use non-interactive editors

```bash
# Configure builtin diff editor (prevents escape sequences)
jj config set --user ui.diff-editor ':builtin'

# Optional: Set a simple text editor
jj config set --user ui.editor 'nano'  # or 'vim', 'code --wait'

# Disable pager for logs
jj config set --user ui.pager 'cat'
```

## Essential Commands Reference

### Daily Workflow

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `jj st` | Check status | Before/after AI work |
| `jj new` | Create sandbox | Before AI agent starts |
| `jj log` | View commit tree | Understand current state |
| `jj squash` | Clean up AI mess | After AI completes task |
| `jj split --interactive=false` | Break apart changes | Avoid if causing terminal issues |
| `jj op log` | See operation history | Debug or recovery |
| `jj op undo` | Undo last operation | Fix mistakes immediately |

### AI Agent Safety Commands

| Command | Purpose | Example Use Case |
|---------|---------|------------------|
| `jj op restore --to=<id>` | Restore to specific state | AI broke everything |
| `jj abandon @` | Abandon current commit | Start over cleanly |
| `jj new @-` | Create sibling revision | Try alternative approach |
| `jj edit @-` | Edit previous commit | Fix AI output in place |
| `jj bookmark create <name>` | Mark important state | Before major AI work |
| `jj describe` | Add commit message | Document AI task completion |

### Git Interop Commands

| Command | Purpose | Notes |
|---------|---------|--------|
| `jj git fetch` | Sync from remote | Before starting work |
| `jj git push` | Push clean commits | Only after squashing |
| `jj git push -c @-` | Push specific commit | Skip messy working copy |
| `jj bookmark list` | Show named branches | Check Git branch state |

## Current OmenDB Setup

### Both Repos Use JJ ✅
```bash
/Users/nick/github/omendb/omendb/        # Public repo
├── .git/                                 # Git (colocated mode)
├── .jj/                                  # Jujutsu tracking
└── omendb/                               # Source code

/Users/nick/github/omendb/omendb-cloud/  # Private repo  
├── .git/                                 # Git (colocated mode)
├── .jj/                                  # Jujutsu tracking
└── docs/                                 # Internal docs
```

## AI Agent Workflow Patterns

### Pattern 1: Sandbox → Squash (Recommended)
```bash
# Before AI work
jj new                          # Create sandbox revision
# AI agent makes changes...
jj squash                       # Merge into parent commit
jj describe                     # Add meaningful commit message
jj git push                     # Share clean result
```

### Pattern 2: Split Workflow (Complex Changes)
```bash
# After AI makes multiple changes
jj split                        # Interactive split into logical commits
# Select files/hunks per commit in TUI
jj describe @- -m "feat: add feature X"
jj describe @ -m "test: add tests for X"
jj git push -c @-               # Push first commit
jj git push -c @               # Push second commit
```

### Pattern 3: Experimental Branches
```bash
# For risky AI experiments
jj bookmark create experiment/ai-refactor
jj new                          # Start experiment
# AI agent works...
# If good:
jj bookmark create main
jj squash
# If bad:
jj abandon @
jj edit @-                      # Back to pre-experiment state
```

## Claude Code Integration

### Automatic Commit Hooks

**Basic Hook Configuration** (`.claude/settings.local.json`):
```json
{
  "hooks": {
    "Start": [{
      "matcher": "",
      "hooks": [{
        "type": "command", 
        "command": "jj new -m 'Claude Code: starting task'"
      }]
    }],
    "Stop": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "jj squash && jj describe -m 'Claude Code: completed task'"
      }]
    }]
  }
}
```

**Advanced Hook with Dynamic Messages**:
```bash
#!/bin/bash
# .claude/hooks/jj-commit.sh
TASK=$(echo "$1" | jq -r '.task // "AI task"')
TIMESTAMP=$(date +"%Y-%m-%d %H:%M")
jj describe -m "Claude Code [$TIMESTAMP]: $TASK"
```

### Claude Code Best Practices
- [MUST] Use `jj new` before each Claude Code session
- [SHOULD] Set up automatic commit hooks
- [MUST] Review changes with `jj diff` before finalizing
- [SHOULD] Use `jj squash` to clean up experimental commits
- [MAY] Create bookmarks for major milestones

## Working Copy States

### Understanding jj States

```
@     Working copy commit (current)
@-    Parent commit (previous)  
@+    Child commit (if exists)
@--   Grandparent commit
```

### State Transitions
```bash
# Current state
jj log -r @:@--

# After jj new
@ (new empty commit)
@- (your previous work)
@-- (previous parent)

# After jj squash
@ (combined changes)  
@- (original grandparent)
```

## Recovery & Safety

### Emergency Recovery Commands

| Situation | Command | Result |
|-----------|---------|---------|
| AI broke everything | `jj op undo` | Undo last operation |
| Need to go back 3 operations | `jj op undo --ignore-working-copy -o 3` | Undo multiple ops |
| Want specific state | `jj op restore --to=abc123` | Restore to exact state |
| Lost important commit | `jj op log` then `jj new <commit-id>` | Recover commit |
| Conflicts everywhere | `jj abandon @` then `jj new @-` | Start fresh |

### Operation Log Usage
```bash
# View all operations
jj op log

# Find specific operation
jj op log | grep "Claude"

# Restore to before AI work
jj op restore --to=$(jj op log | grep "before AI" | cut -d' ' -f1)
```

## File Organization & Cleanup

### jj Repository Structure
```
.jj/                    # jj metadata (never edit)
.git/                   # Git backend (colocated mode)
.claude/                # Claude Code settings
├── settings.local.json # Hook configuration
└── hooks/             # Custom hook scripts
.jjconfig.toml         # User config file
```

### Cleanup Rules
- [MUST] Never commit `.jj/` directory
- [SHOULD] Add `.jj/` to `.gitignore` in colocated repos
- [MUST] Clean up experimental bookmarks regularly
- [SHOULD] Archive old operation log with `jj op abandon`
- [MAY] Use `jj bookmark delete` for temporary bookmarks

## Performance & Optimization

### Large Repository Handling
```bash
# Performance tuning for AI workflows
jj config set --user ui.paginate never     # No pager for automation
jj config set --user core.fsmonitor true   # File system monitoring
jj config set --user git.auto-local-bookmark false  # Reduce branch noise
```

### AI-Optimized Settings
```toml
# ~/.jjconfig.toml
[ui]
default-revset = "@+ | @"  # Show working + children only
diff-editor = "code --wait --diff"
merge-editor = "code --wait"

[git]  
abandon-unreachable-commits = true  # Clean up automatically

[templates]
log = 'commit_id.short() ++ " " ++ description.first_line()'
```

## Common AI Workflows

### Workflow 1: Iterative Development
```bash
# Setup
jj bookmark create feature/ai-assisted

# Each AI iteration
jj new                          # New revision for experiment
# AI makes changes...
jj diff                         # Review AI changes
# If good: continue, if bad: jj abandon @

# When complete
jj squash --from @::@---         # Squash all iterations
jj describe -m "feat: implement X with AI assistance"
```

### Workflow 2: Multiple AI Agents
```bash
# Agent 1: Code generation
jj new -m "AI Agent 1: initial implementation"
# Agent 1 works...

# Agent 2: Tests  
jj new -m "AI Agent 2: add tests"
# Agent 2 works...

# Agent 3: Documentation
jj new -m "AI Agent 3: add docs"  
# Agent 3 works...

# Clean up
jj squash --from @::@---         # Combine all AI work
jj describe -m "Complete feature with tests and docs"
```

### Workflow 3: Conflict Resolution with AI
```bash
# When AI creates conflicts
jj status                        # See conflict markers
# Edit files to resolve conflicts (or ask AI to help)
# No need for git add or continue commands - just save files
jj log                          # Verify resolution
```

## Integration Patterns

### With IDEs and Editors
```bash
# VS Code integration
jj config set --user ui.diff-editor "code --wait --diff"
jj config set --user ui.merge-editor "code --wait"

# For AI code review
alias ai-review='jj diff | claude-code "review this diff"'
alias ai-commit='jj describe -m "$(jj diff | claude-code "generate commit message")"'
```

### With CI/CD
```bash
# Pre-push validation
jj git push --dry-run           # Check what would be pushed
# Only push clean, squashed commits
jj git push -c @-               # Push parent, skip working copy
```

## Quality Gates

### Before AI Agent Work
- [ ] Repository is clean: `jj status` shows no conflicts
- [ ] Latest remote changes: `jj git fetch` 
- [ ] Created sandbox: `jj new`
- [ ] Set bookmark if major work: `jj bookmark create <name>`

### After AI Agent Work  
- [ ] Reviewed changes: `jj diff`
- [ ] Resolved any conflicts: No conflict markers in files
- [ ] Clean commit history: Used `jj squash`/`jj split` appropriately
- [ ] Meaningful commit message: `jj describe`
- [ ] Tests pass: Run test suite
- [ ] No sensitive data: Check for API keys, tokens

### Before Git Push
- [ ] History is clean: `jj log` shows logical commits
- [ ] Working copy is clean: `jj diff` shows intended changes only
- [ ] Remote is up to date: `jj git fetch`
- [ ] Push specific commit: `jj git push -c @-` (avoid pushing working copy)

## Troubleshooting Guide

### Common Issues with AI Agents

| Problem | Symptoms | Solution |
|---------|----------|----------|
| AI created mess | Working copy has random changes | `jj abandon @` then `jj new @-` |
| Conflicts after AI work | Conflict markers in files | Edit files, save (no add/commit needed) |
| Lost important change | Can't find previous state | `jj op log` then `jj op restore --to=<id>` |
| Too many commits | Linear chain of AI commits | `jj squash --from @::@---` |
| Wrong commit message | Commit has bad description | `jj describe -m "new message"` |
| Pushed working copy | Pushed experimental changes | `jj git push -f -c @-` to fix |

### Debug Commands
```bash
# Understand current state
jj log -r @::@---               # Show recent commits
jj op log | head -10            # Show recent operations
jj status --no-pager            # Current working copy state

# Find lost work
jj op log | grep -i "claude"    # Find AI operations
jj log --abandoned              # Show abandoned commits
```

## Advanced AI Patterns

### Multi-Agent Coordination
```bash
# Agent A: Backend
jj new -m "Agent A: backend implementation"
# Agent A works...

# Agent B: Frontend (parallel)
jj new @-- -m "Agent B: frontend implementation"  
# Agent B works...

# Combine work
jj rebase -d @ @-               # Rebase frontend onto backend
jj squash                       # Clean up
```

### Checkpoint Pattern
```bash
# Create recovery points during long AI sessions
jj bookmark create checkpoint/$(date +%s)
# AI continues work...
# If things go wrong:
jj new checkpoint/1234567890    # Return to checkpoint
```

### Code Review with AI
```bash
# Prepare clean commits for review
jj squash --from @::@---
jj split                        # Break into logical pieces
jj describe @- -m "feat: implement feature X"
jj describe @ -m "test: add comprehensive tests"

# Generate review request
jj diff @- | claude-code "create PR description for this change"
```

## Configuration Templates

### Personal Config (`~/.jjconfig.toml`)
```toml
[user]
name = "Your Name"
email = "your.email@domain.com"

[ui]
default-revset = "@+ | @"  # Focus on current work
paginate = "never"         # Better for automation
diff-format = "colored-words"

[git]
abandon-unreachable-commits = true
auto-local-bookmark = false    # Reduce branch noise

[aliases]
# AI-friendly aliases  
sandbox = ["new"]
cleanup = ["squash"]
checkpoint = ["bookmark", "create"]
```

### Project Config (`.jj/config.toml`)
```toml
[revset-aliases]
"ai-commits" = "description(glob:'Claude*') | description(glob:'AI*')"
"today" = "committer_date(after:'today')"

[templates]
commit_summary = '''
{commit_id.short()} {description.first_line()}
{if(conflict, "CONFLICT ")}
'''
```

## Hook Configurations

### Basic Claude Code Hook
```json
{
  "hooks": {
    "Start": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "jj new"
      }]
    }],
    "Stop": [{
      "matcher": "",  
      "hooks": [{
        "type": "command",
        "command": "jj describe -m 'Claude Code: auto-commit'"
      }]
    }]
  }
}
```

### Advanced Hook with Context
```bash
#!/bin/bash
# .claude/hooks/jj-smart-commit.sh
set -e

INPUT=$(cat)
TASK=$(echo "$INPUT" | jq -r '.task // "AI task"')
FILES_CHANGED=$(jj diff --name-only | wc -l)
TIMESTAMP=$(date +"%Y-%m-%d %H:%M")

if [ "$FILES_CHANGED" -gt 10 ]; then
    # Large change - create checkpoint first
    jj bookmark create "checkpoint/$(date +%s)"
fi

# Smart commit message based on changes
if [ "$FILES_CHANGED" -eq 1 ]; then
    FILE=$(jj diff --name-only)
    MSG="Claude Code: modify $FILE"
else
    MSG="Claude Code [$TIMESTAMP]: $TASK ($FILES_CHANGED files)"
fi

jj describe -m "$MSG"
```

## Workflow Decision Trees

### Pre-AI Work Decision Tree
```
Starting AI task?
├─ Small experiment (< 5 files) → jj new
├─ Major feature → jj new + jj bookmark create feature/name  
├─ Risky refactor → jj bookmark create backup/$(date +%s)
└─ Bug fix → jj new -m "fix: preparing AI bug fix"
```

### Post-AI Work Decision Tree  
```
AI completed task?
├─ Single logical change → jj squash + jj describe
├─ Multiple features → jj split (interactive)
├─ Experimental/messy → jj abandon @ + start over
└─ Perfect as-is → jj describe only
```

### Recovery Decision Tree
```
Something went wrong?
├─ Last operation was bad → jj op undo
├─ Need specific old state → jj op log + jj op restore --to=<id>
├─ Working copy is corrupted → jj abandon @ + jj new @-
└─ Lost commits → jj log --abandoned + jj new <commit-id>
```

## Performance Optimizations

### For Large Repositories
```bash
# Speed up jj operations
jj config set --user core.fsmonitor true
jj config set --user ui.paginate never
jj config set --user git.auto-local-bookmark false

# Reduce operation log size
jj op abandon $(jj op log --limit 100 | tail -1 | cut -d' ' -f1)
```

### For AI-Heavy Workflows
```bash
# Batch operations for efficiency
jj config set --user snapshot.max-new-file-size 10MB  # Skip large files
jj config set --user core.watchman true              # File watching

# Template for concise logs
jj config set --user templates.log_oneline \
  'commit_id.short() ++ " " ++ description.first_line().substr(0, 60)'
```

## Safety & Backup Strategies

### Backup Patterns
```bash
# Daily backup bookmark
jj bookmark create "backup/$(date +%Y%m%d)"

# Before major AI work  
jj bookmark create "pre-ai/$(date +%s)"

# Archive old operation history
jj op abandon $(jj op log --limit 1000 | tail -1 | awk '{print $1}')
```

### Recovery Testing
```bash
# Test recovery workflow (safe to run)
jj bookmark create test-recovery
jj new -m "test commit"
echo "test" > test.txt
jj op log | head -5             # Note operation ID
jj op undo                      # Should undo file creation
[ ! -f test.txt ] && echo "Recovery works!"
```

## Anti-Patterns (Avoid These)

### Git Muscle Memory Mistakes
```bash
# BAD: Git-style workflow
git add .                       # Not needed in jj
git commit -m "message"         # Use jj describe instead
git checkout -b branch          # Use jj bookmark create instead

# GOOD: jj native workflow  
jj describe -m "message"        # Set commit description
jj bookmark create feature     # Create named pointer
```

### Over-Committing  
```bash
# BAD: Creating too many commits
jj new -m "AI: step 1"
jj new -m "AI: step 2"  
jj new -m "AI: step 3"
# Results in messy linear history

# GOOD: Sandbox then squash
jj new                          # Single sandbox
# AI does all work...
jj squash                       # Clean single commit
jj describe -m "Complete feature implementation"
```

### Ignoring Operation Log
```bash
# BAD: Not using operation log for debugging
# Something went wrong, start over from scratch

# GOOD: Use operation log for recovery
jj op log | grep "before AI"    # Find good state
jj op restore --to=<good-state> # Restore precisely
```

## Working with Subagents

### For Main Agent (Claude Code)
1. **Create sandbox**: `jj new` before major work
2. **Let auto-track work**: Don't worry about commits during work
3. **Clean up after**: `jj squash` to combine related changes
4. **Push when ready**: `jj git push` after squashing

### For Subagents
1. **Read-only is safe**: Subagents doing research don't affect JJ
2. **Write operations**: Main agent should `jj new` before launching write subagents
3. **Parallel agents**: Each can work in different directories, JJ tracks all
4. **Cleanup**: Main agent squashes all subagent work into logical commits

### Example: Memory Optimization with Multiple Agents
```bash
# 1. Main agent creates sandbox
jj new -m "Memory optimization work"

# 2. Launch subagents
- Quantization fix agent (writes to core/)
- Pre-allocation agent (writes to algorithms/)  
- Test fix agent (writes to tests/)

# 3. All changes auto-captured by JJ

# 4. Main agent reviews and squashes
jj squash -m "fix: reduce memory usage by 85%"

# 5. Push to GitHub
jj git push
```

## Benefits Over Git for AI Work

| Scenario | Git | JJ |
|----------|-----|-----|
| Agent forgets to commit | Work lost | Auto-saved |
| Agent makes bad change | Complex revert | `jj op undo` |
| Multiple agents working | Merge conflicts | Auto-tracked |
| Messy commit history | Permanent | `jj squash` cleans |
| Experimentation | Branch juggling | Sandbox revisions |

## Best Practices Summary

### For AI Agents
1. **Always start with `jj new`** - Create sandbox for AI work
2. **Review before finalizing** - Use `jj diff` to check AI changes
3. **Clean up history** - Use `jj squash` for messy AI commits
4. **Meaningful descriptions** - Use `jj describe` with clear messages  
5. **Test recovery** - Practice using `jj op undo` and `jj op restore`

### For Human Developers
1. **Trust the safety net** - jj makes it safe to experiment
2. **Use operation log** - `jj op log` is your debug tool
3. **Embrace working copy commits** - No need for manual staging
4. **Bookmark important states** - Mark milestones for easy return
5. **Keep Git interop clean** - Only push squashed, clean commits

## Command Aliases for AI Work

Add to `~/.jjconfig.toml`:
```toml
[aliases]
# AI workflow shortcuts
sandbox = ["new"]
cleanup = ["squash"] 
ai-split = ["split"]
ai-log = ["log", "-r", "description(glob:'Claude*') | description(glob:'AI*')"]
safe-push = ["git", "push", "-c", "@-"]
checkpoint = ["bookmark", "create"]
undo-ai = ["op", "undo"]
```

## Migration from Git

### Gradual Migration
```bash
# Start with existing Git repo
cd existing-repo
jj git init --colocate          # Add jj to existing Git repo

# Use both temporarily
git status                      # Still works
jj status                       # New workflow available

# Switch Claude Code to use jj hooks when ready
```

### Full Migration
```bash
# Clone existing repo with jj
jj git clone <url> <dir>

# Or convert existing
cd git-repo
jj git init --git-repo=.
```

## Checklist for AI Agent Integration

### Initial Setup
- [ ] jj is installed: `jj version` works
- [ ] Repository initialized: `.jj/` directory exists
- [ ] Claude Code hooks configured: `.claude/settings.local.json` exists  
- [ ] User config set: `~/.jjconfig.toml` has basic settings
- [ ] Git interop tested: `jj git push --dry-run` works

### Per-Session Workflow
- [ ] Start with clean state: `jj status` shows no conflicts
- [ ] Create sandbox: `jj new` before AI work
- [ ] Monitor AI changes: Regular `jj diff` checks
- [ ] Clean up afterward: `jj squash` or `jj split` as needed
- [ ] Test before sharing: Run tests after squash
- [ ] Push clean history: Use `jj git push -c @-`

### Recovery Readiness
- [ ] Know operation log: Practice `jj op log` 
- [ ] Test undo: Practice `jj op undo`
- [ ] Bookmark important states: Use `jj bookmark create` 
- [ ] Understand working copy: Know difference between `@` and `@-`
- [ ] Recovery plan: Know how to abandon and restart

## Quick Reference Card

### Essential Commands (Print This)
```
# Sandbox workflow
jj new              # Create experiment space
jj squash           # Clean up AI mess
jj describe         # Add commit message
jj abandon @        # Delete current commit

# Recovery commands  
jj op log           # See all operations
jj op undo          # Undo last operation
jj op restore --to= # Restore specific state

# Status & review
jj st               # Working copy status  
jj log              # Commit tree
jj diff             # Current changes
jj diff @-          # Changes vs parent

# Git interop
jj git fetch        # Update from remote
jj git push -c @-   # Push parent commit
jj bookmark create  # Create named branch
```

### Emergency Commands
```
# Oh no, AI broke everything!
jj op undo                      # Undo last operation
jj op restore --to=<good-id>    # Nuclear option
jj abandon @                    # Abandon working copy
jj new @-                       # Start fresh from parent
```

## Recommendation for AI Development

**Keep using JJ!** It's ideal for AI development because:
1. **Safety**: Can't lose work - everything is auto-tracked
2. **Cleanliness**: Squash messy AI commits into logical changes
3. **Flexibility**: Experiment freely with sandbox revisions
4. **Recovery**: Undo anything with operation log

The combination of JJ's automatic tracking and AI agents' prolific output is perfect. Agents can focus on solving problems while JJ handles version control automatically.

### Current Status
- ✅ Both OmenDB repos have JJ configured
- ✅ Colocated with Git for GitHub compatibility
- ✅ Auto-tracking all changes
- ✅ Operation log providing safety net

### Watch Out For
- Terminal escape sequences (fixed with `jj config set --user ui.diff-editor ':builtin'`)
- Remember to squash before pushing
- Don't let subagents run git commands directly

---
*Complete jj + AI Agent workflow guide. For project-specific patterns, check project's jj configuration and Claude Code hooks.*