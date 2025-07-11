# JSON Task Management for AI Agents

**Simple, structured task tracking that AI agents can reliably update**

## Why JSON Over Markdown

- ✅ **No merge conflicts** - AI agents can update without conflicts
- ✅ **Structured queries** - Easy filtering with `jq`
- ✅ **Programmatic updates** - Direct JSON manipulation
- ✅ **Real-time validation** - Catch errors before commit

## Setup (5 minutes)

### 1. Create Structure
```bash
mkdir -p docs/tasks
touch docs/tasks/tasks.json docs/tasks/completed.json
```

### 2. Initialize tasks.json
```json
{
  "priorities": {
    "next_sequence": [],
    "current_focus": "Initial project setup"
  },
  "project": {
    "name": "Your Project",
    "description": "Brief description",
    "version": "0.1.0"
  },
  "tasks": {}
}
```

### 3. Add Makefile Targets (Optional but Recommended)
```makefile
validate-tasks:
	@jq empty docs/tasks/tasks.json && echo "✅ Valid" || echo "❌ Invalid"

current-tasks:
	@jq -r '.tasks | to_entries | map(select(.value.status == "in_progress")) | .[] | "- \(.key): \(.value.title)"' docs/tasks/tasks.json

progress:
	@jq -r '.tasks | to_entries | group_by(.value.status) | map("\(.value.status): \(length) tasks") | .[]' docs/tasks/tasks.json
```

## Task Schema

```json
{
  "TASK-001": {
    "title": "Implement feature X",
    "status": "pending",
    "priority": "high", 
    "description": "Detailed description",
    "acceptance_criteria": [
      "Implement core functionality",
      "Add tests",
      "Update documentation"
    ],
    "files": ["src/module.rs"],
    "notes": "Progress updates go here"
  }
}
```

## AI Agent Workflow

### 1. Check Current Work
```bash
jq '.priorities.next_sequence[] as $id | .tasks[$id] | select(.status != "completed")' docs/tasks/tasks.json
```

### 2. Start Task  
```bash
jq '.tasks["TASK-001"].status = "in_progress"' docs/tasks/tasks.json > tmp.json && mv tmp.json docs/tasks/tasks.json
```

### 3. Add Progress Notes
```bash
jq '.tasks["TASK-001"].notes = "Implemented basic structure"' docs/tasks/tasks.json > tmp.json && mv tmp.json docs/tasks/tasks.json
```

### 4. Complete Task
```bash
jq '.tasks["TASK-001"].status = "completed"' docs/tasks/tasks.json > tmp.json && mv tmp.json docs/tasks/tasks.json
```

## CLAUDE.md Integration

Add to your CLAUDE.md:
```markdown
## Task Management
**CRITICAL**: Use `docs/tasks/tasks.json` as SINGLE SOURCE OF TRUTH

### Workflow
1. Check priorities: `jq '.priorities.next_sequence' docs/tasks/tasks.json`
2. Mark in progress BEFORE starting work
3. Update status IMMEDIATELY after completion
4. Add notes for progress tracking
```

## Quality Gates

Before marking any task complete:
- ✅ All acceptance criteria met
- ✅ Tests pass
- ✅ Code formatted and linted  
- ✅ Documentation updated

## Pro Tips

- **Start Simple**: Begin with basic task structure, add fields as needed
- **Update Real-Time**: Change status immediately, don't batch updates
- **Use Validation**: Add git pre-commit hook to validate JSON
- **Keep Notes**: Use `notes` field for progress updates
- **Trust the System**: Never create duplicate task lists elsewhere

This system scales from solo development to team projects while maintaining simplicity and reliability for AI agents.