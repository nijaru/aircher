# Task Completion Checklist

## Before Starting a Task
- [ ] Task status set to `in_progress` in `docs/tasks/tasks.json`
- [ ] Required documentation reviewed (see task-to-documentation mapping in CLAUDE.md)
- [ ] Dependencies identified and completed
- [ ] Required files and directories exist

## During Implementation
- [ ] Follow architecture patterns from `docs/core/MASTER_SPEC.md`
- [ ] Implement error handling for all failure modes
- [ ] Add logging with appropriate levels
- [ ] Write tests as you implement functionality
- [ ] Update documentation for any architectural changes

## Before Marking Task Complete
- [ ] All acceptance criteria met
- [ ] Code quality checks pass (`cargo clippy`, `cargo test`, `cargo fmt`)
- [ ] Manual testing performed for user-facing features
- [ ] Performance implications considered and documented
- [ ] Security implications reviewed

## After Task Completion
- [ ] Task status updated to `completed` in `docs/tasks/tasks.json`
- [ ] Task moved to `docs/tasks/completed.json` if archival needed
- [ ] Related documentation updated if necessary
- [ ] Next dependent tasks identified and prioritized

## LLM Provider Tasks (Phase 2)
- [ ] Provider trait properly implemented
- [ ] Authentication mechanism working
- [ ] Streaming responses integrated with TUI
- [ ] Error handling covers all API error types
- [ ] Token counting accurate (if applicable)
- [ ] Rate limiting respected
- [ ] Connection health checks implemented

## TUI Integration Tasks
- [ ] Component renders properly in all terminal sizes
- [ ] Keyboard navigation works correctly
- [ ] Responsive design maintains usability
- [ ] State management handles edge cases
- [ ] Performance acceptable for real-time updates

## Database Tasks
- [ ] Migrations run successfully
- [ ] Queries are type-safe with sqlx
- [ ] Transaction boundaries are appropriate
- [ ] Connection pooling configured correctly
- [ ] Data integrity constraints enforced