# Pre-Commit Checklist

## Code Quality
- [ ] `cargo clippy` passes without warnings
- [ ] `cargo test` passes all tests
- [ ] `cargo fmt` has been run (code is properly formatted)
- [ ] No `TODO` or `FIXME` comments in committed code
- [ ] No debug print statements (`println!`, `dbg!`) in committed code

## Architecture Compliance
- [ ] Changes follow patterns defined in `docs/core/MASTER_SPEC.md`
- [ ] New traits implement `Send + Sync` where appropriate
- [ ] Database operations use proper transaction handling
- [ ] Error handling follows project error types
- [ ] Async code uses proper tokio patterns

## Security
- [ ] No hardcoded secrets, API keys, or credentials
- [ ] No `unsafe` code without thorough documentation
- [ ] Input validation added for user-facing functions
- [ ] File permissions properly set for sensitive operations

## Documentation
- [ ] Public APIs have proper documentation comments
- [ ] Complex logic has inline comments explaining the "why"
- [ ] Changes to architecture are reflected in relevant docs
- [ ] Task status updated in `docs/tasks/tasks.json`

## Testing
- [ ] Unit tests added for new functionality
- [ ] Integration tests pass for affected components
- [ ] Error cases have test coverage
- [ ] Performance-critical code has basic benchmarks

## LLM Provider Integration (Phase 2 Specific)
- [ ] Provider interface properly implemented
- [ ] Streaming responses handle cancellation
- [ ] API errors mapped to project error types
- [ ] Token counting implemented where applicable
- [ ] Rate limiting handled appropriately