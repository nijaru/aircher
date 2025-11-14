# Aircher TODO

## Phase 6: Benchmarking & Optimization (ACTIVE)

### Terminal-Bench Integration
- [ ] Set up Terminal-Bench evaluation harness
- [ ] Run baseline evaluation (expect 35-45%)
- [ ] Analyze failure patterns
- [ ] Optimize based on findings
- [ ] Re-run to validate improvements

### SWE-bench Sample (Optional)
- [ ] Run SWE-bench Verified sample (50 tasks)
- [ ] Analyze performance vs SOTA (75%)
- [ ] Document findings

### Optimization & Polish
- [ ] Address benchmark failures
- [ ] Tune prompts and workflows
- [ ] Optimize memory queries for performance
- [ ] Validate 60% tool reduction claim
- [ ] Final testing and bug fixes

**Phase 6 Success**: Terminal-Bench >43.2% (beat Claude Code), tool reduction validated

## Backlog (Post Phase 6)

### Toad Integration
- [ ] Wait for Toad open source release
- [ ] Test Aircher with Toad frontend
- [ ] Optimize for Toad-specific features
- [ ] Update documentation

### Advanced Features
- [ ] Undo subsystem (git-based time travel)
- [ ] Smart context compaction (beyond pruning)
- [ ] Cross-session pattern learning improvements
- [ ] GUI automation capabilities (if needed)
- [ ] MCP server support (extensibility)

### Deferred from Phase 5
- [ ] Streaming responses (complex SSE - implement if benchmarks show need)
- [ ] Integration tests with real ACP client (formalize if needed)
- [ ] Performance benchmarks (<100ms p95 - measure during Terminal-Bench)

### Technical Debt
- [ ] SQLite cleanup: Remove unused tool_calls table
- [ ] Pydantic models for LLM output validation
- [ ] Pydantic Logfire for observability (nice-to-have)

### Documentation & Release (Future)
- [ ] User documentation (docs/)
- [ ] API documentation (docstrings + mkdocs)
- [ ] Tutorial videos/examples
- [ ] Contribution guidelines
- [ ] Prepare for 0.1.0 release

## Notes

- Focus on Phase 6 (benchmarking) - biggest validation of SOTA claims
- Don't skip testing - comprehensive tests prevent regression
- Benchmark early to validate claims with empirical data
- Keep ai/STATUS.md updated with progress

## References

- **Implementation Plan**: ai/PLAN.md (phase dependencies, architecture)
- **Memory Architecture**: ai/research/memory-system-architecture.md
- **Sub-Agent Patterns**: ai/research/crush-subagent-architecture.md
- **Competitive Analysis**: ai/research/competitive-analysis-2025.md
- **Benchmarking Strategy**: ai/research/benchmark-integration-plan.md
