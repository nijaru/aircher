# Aircher Task Management System

Revolutionary JSON-based task tracking system optimized for AI-assisted development.

## Overview

This directory contains the single source of truth for all Aircher project tasks, replacing traditional Markdown-based task tracking with structured JSON data that enables programmatic updates and queries.

## File Structure

```
docs/tasks/
├── README.md          # This file - task system guide
├── tasks.json         # Active tasks (SINGLE SOURCE OF TRUTH)
├── completed.json     # Completed task archive
└── templates/         # Task templates for common patterns
```

## Key Benefits Over Markdown

- **Programmatic Updates**: AI agents can directly update task status without merge conflicts
- **Structured Queries**: Easy filtering by status, priority, phase, or sprint
- **Historical Tracking**: Complete audit trail of task progression
- **Metrics Integration**: Automatic progress calculation and reporting
- **No Merge Conflicts**: JSON structure prevents common Git merge issues

## Task Schema

### Core Task Structure

```json
{
  "TASK-ID": {
    "title": "Human-readable task name",
    "phase": "Phase N: Description",
    "status": "pending|in_progress|blocked|completed",
    "priority": "critical|high|medium|low",
    "description": "Detailed task description",
    "sprint": "immediate|medium|long_term", // Optional
    "acceptance_criteria": [
      "Criterion 1: Specific requirement",
      "Criterion 2: Another requirement"
    ],
    "files": ["src/file1.rs", "src/file2.rs"],
    "notes": "Optional progress notes"
  }
}
```

### Project Metadata

```json
{
  "project": {
    "name": "Project Name",
    "description": "Brief project description",
    "version": "Current version",
    "technology_stack": ["Tech1", "Tech2"],
    "current_phase": "Current development phase"
  },
  "metrics": {
    "code_statistics": {
      "total_go_files": number,
      "lines_of_code": number,
      "test_coverage": percentage
    },
    "phase_progress": {
      "phase_1_foundation": percentage,
      "phase_2_intelligence": percentage
    }
  }
}
```

## Status Definitions

| Status | Symbol | Description |
|--------|--------|-------------|
| `pending` | ❌ | Task not yet started |
| `in_progress` | 🚧 | Task currently being worked on |
| `blocked` | 🔄 | Task blocked by dependencies |
| `completed` | ✅ | Task fully implemented and tested |

## Priority Levels

- **Critical**: Blocks other work, must be completed immediately
- **High**: Important for current sprint/milestone
- **Medium**: Planned for near-term completion
- **Low**: Nice-to-have, scheduled for future sprints

## Task ID Naming Convention

- **PHASE[N]-[###]**: Phase-specific tasks (e.g., `PHASE1-001`, `PHASE2-003`)
- **SPRINT-[###]**: Current sprint tasks (e.g., `SPRINT-001`)
- **FIX-[###]**: Bug fixes and critical issues
- **REFACTOR-[###]**: Code refactoring and improvements

## AI Agent Workflow

### 1. Reading Tasks
```bash
# Get current high-priority tasks
jq '.tasks | to_entries | map(select(.value.priority == "critical" and .value.status != "completed"))' tasks.json

# Get tasks for current sprint
jq '.current_sprint.immediate_tasks[] as $id | .tasks[$id]' tasks.json
```

### 2. Updating Task Status
```bash
# Mark task as in progress
jq '.tasks["TASK-ID"].status = "in_progress"' tasks.json > tmp.json && mv tmp.json tasks.json

# Add progress notes
jq '.tasks["TASK-ID"].notes = "Progress update"' tasks.json > tmp.json && mv tmp.json tasks.json

# Or use Makefile helpers
make current-tasks  # Show active work
make progress      # Show overall status
```

### 3. Task Completion
When a task is completed:
1. Update status to `completed`
2. Move to `completed.json` with completion timestamp
3. Update project metrics
4. Remove from active sprint if applicable

## Project Metrics Tracking

The system automatically tracks:
- **Code Statistics**: Files, lines of code, test coverage
- **Phase Progress**: Completion percentage per development phase
- **Sprint Velocity**: Tasks completed per sprint
- **Quality Gates**: Test coverage, lint status, performance metrics

## Sprint Management

### Current Sprint Structure
```json
{
  "current_sprint": {
    "name": "Sprint Name",
    "duration": "2 weeks",
    "focus": "Sprint objective",
    "immediate_tasks": ["TASK-ID-1", "TASK-ID-2"],
    "medium_priority": ["TASK-ID-3"],
    "long_term": ["TASK-ID-4"]
  }
}
```

### Sprint Planning
1. Review completed tasks and update metrics
2. Select high-priority tasks for immediate focus
3. Balance workload across different project areas
4. Update sprint metadata and task assignments

## Quality Gates

Tasks must meet quality criteria before completion:

### Code Quality
- [ ] Test coverage meets minimum threshold
- [ ] Linting passes without errors
- [ ] Security scan passes
- [ ] Code review completed

### User Experience
- [ ] Response time under 200ms
- [ ] Error handling and recovery
- [ ] Cross-platform compatibility
- [ ] Documentation updated

## Integration with Documentation

This task system integrates with the broader documentation structure:

- **Architecture Tasks** → `docs/architecture/` specifications
- **Development Tasks** → `docs/development/` guides
- **Validation Tasks** → `docs/reference/validation/` checklists

## Best Practices

### For AI Agents
1. **Always Read First**: Check current task status before starting work
2. **Update Incrementally**: Mark acceptance criteria as completed progressively
3. **Maintain Metrics**: Update code statistics after significant changes
4. **Document Blockers**: Record dependencies and blocking issues
5. **Archive Completed**: Move finished tasks to completed.json

### For Human Developers
1. **Review JSON Structure**: Understand the schema before making changes
2. **Use Validation**: Validate JSON structure after manual edits
3. **Update Metrics**: Keep project statistics current
4. **Plan Sprints**: Regularly review and update sprint priorities

## Common Queries

### Using Makefile (Recommended)
```bash
make current-tasks   # Show in-progress and high-priority pending
make progress       # Show status summary and next priorities
make validate-tasks # Check JSON is valid
make new-task      # Create new task interactively
```

### Using jq Directly
```bash
# Find tasks by status
jq '.tasks | to_entries | map(select(.value.status == "in_progress"))' tasks.json

# Get high priority tasks
jq '.tasks | to_entries | map(select(.value.priority == "critical" or .value.priority == "high"))' tasks.json

# Calculate phase progress
jq '.tasks | to_entries | group_by(.value.phase) | map({phase: .[0].value.phase, total: length, completed: map(select(.value.status == "completed")) | length})' tasks.json
```

## Migration from Markdown

When migrating from traditional Markdown task lists:

1. **Extract Tasks**: Convert each task item to JSON structure
2. **Add Metadata**: Include priority, phase, and acceptance criteria
3. **Preserve Context**: Maintain task descriptions and requirements
4. **Update References**: Change documentation links to reference JSON structure
5. **Validate Structure**: Ensure JSON is valid and follows schema

## Troubleshooting

### JSON Validation Errors
```bash
# Validate JSON structure
jq empty tasks.json && echo "Valid JSON" || echo "Invalid JSON"

# Pretty-print for debugging
jq . tasks.json
```

### Merge Conflicts
JSON structure minimizes merge conflicts, but if they occur:
1. Choose the most recent status updates
2. Preserve all acceptance criteria
3. Validate final JSON structure
4. Update metrics to reflect current state

---

**Remember**: This JSON-based system is the SINGLE SOURCE OF TRUTH for all task management. Never create duplicate task lists in other files.