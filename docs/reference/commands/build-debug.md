# Essential Commands Reference

## Build Commands

```bash
cargo build --release  # Build aircher binary
cargo test             # Run tests  
cargo clippy           # Run linting
cargo clean            # Clean artifacts
```

## Task Management

```bash
# View current tasks
jq '.tasks | to_entries | map(select(.value.status == "pending"))' docs/tasks/tasks.json

# Update task status
jq '.tasks["TASK-ID"].status = "in_progress"' docs/tasks/tasks.json

# View current sprint
jq '.current_sprint' docs/tasks/tasks.json
```

## CLI Commands (Implemented)

```bash
./aircher                    # Interactive mode
./aircher "prompt"           # Single query
./aircher -c                 # Continue last conversation
./aircher -p "query"         # One-shot mode
./aircher config             # Configuration
./aircher login [provider]   # API key setup
./aircher version            # Version info
```

## Migration Tracking

```bash
# Analyze Go codebase
find . -name "*.go" -not -path "./vendor/*" | wc -l

# Check migration plan status  
cat docs/core/RUST_MIGRATION_PLAN.md | grep -A5 "Phase [0-9]"
```

## Quick Debugging

```bash
# Test basic functionality
go run cmd/aircher/main.go version

# Check config location
ls ~/.config/aircher/

# Database inspection (if exists)
find ~/.local/share/aircher/ -name "*.db" 2>/dev/null
```

## Development Workflow

```bash
# Daily workflow
cargo test && cargo clippy && cargo build --release

# Update task progress in docs/tasks/tasks.json
# Follow docs/core/DEVELOPER_GUIDE.md patterns
# Reference docs/core/MASTER_SPEC.md for architecture
```
