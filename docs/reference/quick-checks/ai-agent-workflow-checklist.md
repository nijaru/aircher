# AI Agent Workflow Checklist

## Session Start
- [ ] Check `docs/tasks/tasks.json` for current priorities
- [ ] Review `next_sequence` array for immediate tasks
- [ ] Load task-specific documentation using mapping table in CLAUDE.md
- [ ] Verify current working directory is project root

## Task Planning
- [ ] Understand task description and acceptance criteria
- [ ] Identify required files and dependencies
- [ ] Review supporting documentation
- [ ] Plan implementation approach following project patterns

## Implementation
- [ ] Update task status to `in_progress`
- [ ] Follow architecture patterns from `docs/core/MASTER_SPEC.md`
- [ ] Use existing utilities and traits where possible
- [ ] Implement proper error handling
- [ ] Add appropriate logging

## Code Quality
- [ ] Run `cargo clippy` and fix all warnings
- [ ] Run `cargo test` and ensure all tests pass
- [ ] Run `cargo fmt` to format code
- [ ] Check for debug prints and TODO comments
- [ ] Verify no secrets or credentials in code

## Validation
- [ ] Manual testing of implemented functionality
- [ ] Verification against acceptance criteria
- [ ] Performance check for critical paths
- [ ] Security review for user-facing code

## Task Completion
- [ ] All acceptance criteria met
- [ ] Task status updated to `completed`
- [ ] Documentation updated if architectural changes made
- [ ] Next task identified and prioritized

## Error Recovery
- [ ] Compilation errors: Check `docs/development/troubleshooting/`
- [ ] Architecture questions: Reference `docs/core/MASTER_SPEC.md`
- [ ] Pattern confusion: Check `docs/reference/patterns/`
- [ ] Test failures: Review `docs/reference/validation/`

## Context Management
- [ ] Only load task-relevant documentation
- [ ] Use tool-based access for non-core docs
- [ ] Minimize token usage while maintaining effectiveness
- [ ] Reference specific file paths with line numbers when helpful

## Communication
- [ ] Provide clear, concise updates
- [ ] Reference specific files and line numbers
- [ ] Explain architectural decisions when relevant
- [ ] Document any blockers or issues discovered