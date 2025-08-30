# Documentation Standards & Organization Guide
*General-purpose guide for project documentation*

## Core Principles

### 1. Single Source of Truth
Every piece of information lives in EXACTLY ONE place. Never duplicate facts across files.

### 2. Clear Hierarchy
```
README.md           # Public introduction
AGENTS.md           # AI agent context (if using AI)
STATUS.md           # Current state (what is)
TECH_SPEC.md        # Architecture (how it works)
TODO.md             # Tasks (what needs doing)
DECISIONS.md        # History (why decisions were made)
```

### 3. Token Efficiency
- Reference, don't duplicate: `See STATUS.md#performance`
- Use anchors for sections: `#known-issues`
- Keep summaries in AGENTS.md minimal
- Full details in dedicated files

## File Organization

### What Goes Where

| Information Type | Location | Example |
|-----------------|----------|---------|
| Quick context | AGENTS.md | "Project overview for AI agents" |
| Current metrics | STATUS.md | "Performance, issues, state" |
| Known bugs | STATUS.md#known-issues | "Active issues and workarounds" |
| Architecture | TECH_SPEC.md#architecture | "System design and structure" |
| Code examples | TECH_SPEC.md | "Implementation patterns" |
| Configuration | CONFIG.md or settings/ | "Environment and settings" |
| Testing guide | TESTING.md | "How to run tests" |
| Active tasks | TODO.md | "Current priorities" |
| Past decisions | DECISIONS.md | "Why things were done" |
| Dev history | CHANGELOG.md | "Version history" |
| Navigation map | docs/index.md | "Documentation structure" |

### When to Create New Files

**Create new file when:**
- New major feature spec (e.g., ADMIN_DASHBOARD_SPEC.md)
- Distinct domain (e.g., GPU_STRATEGY.md)
- Over 500 lines would be added to existing file

**Update existing file when:**
- Information logically belongs there
- Under 100 lines of new content
- Updating metrics, status, or decisions

### File Naming Convention

```
UPPERCASE.md        # Core docs (STATUS, TECH_SPEC)
FEATURE_NAME.md     # Feature specs (ADMIN_DASHBOARD_SPEC)
category/DETAIL.md  # Subdirectory for related files
```

## Maintenance Rules

### Updating Information

1. **Find current location**: Check REFERENCE.md
2. **Update in place**: Don't create duplicates
3. **Update references**: If moving info, update all links
4. **Archive if major change**: Move old version to archive/

### Archiving vs Deleting

**Archive when:**
- Document has historical value
- Contains decisions/rationale
- Might need to reference later
- Major refactor/rewrite

**Delete when:**
- Temporary notes
- Duplicate content
- No historical value
- Correcting typos only

### Date Standards

Always use ISO format: `2025-08-23`
```bash
date +"%Y-%m-%d"  # Command to get current date
```

### Version Standards

**Semantic Versioning**: MAJOR.MINOR.PATCH
- MAJOR: Breaking API changes
- MINOR: New features, backward compatible
- PATCH: Bug fixes, backward compatible

**Pre-1.0**: API may change
**Post-1.0**: API stability guaranteed

## Documentation Workflow

### For Performance Updates
1. Test and measure
2. Update STATUS.md#performance-metrics
3. Add decision to DECISIONS.md if significant
4. Update AGENTS.md if major milestone

### For Bug Fixes
1. Fix the bug
2. Update STATUS.md#known-issues
3. Remove from TODO.md
4. Document solution in DECISIONS.md if notable

### For Architecture Changes
1. Document decision in DECISIONS.md FIRST
2. Update TECH_SPEC.md with details
3. Update STATUS.md if impacts current state
4. Update AGENTS.md summary if major

## AI Agent Optimization

### File Inclusion Strategy

**Minimal context (fast responses):**
```bash
@AGENTS.md  # Always, provides base context
```

**Standard work:**
```bash
@AGENTS.md
@docs/STATUS.md  # Add current state
```

**Complex work:**
```bash
@AGENTS.md
@docs/STATUS.md
@docs/TECH_SPEC.md  # Add technical details
@docs/TODO.md       # Add task context
```

### Token-Saving Tips

1. **Use section anchors**: `@docs/STATUS.md#performance-metrics`
2. **Reference don't copy**: "See TECH_SPEC.md for details"
3. **Summarize in AGENTS.md**: Keep details in dedicated files
4. **Archive old content**: Don't keep outdated info in active files

## TUI Tool Status Lines

This project renders tool execution in the chat as compact, single‑line status messages to keep the terminal UI readable.

- Format: `symbol tool target — state/summary`
- Running: `🔧 read_file Cargo.toml — running…`
- Success: `✓ read_file Cargo.toml — 120 lines`
- Error: `✗ run_command cargo test — exit 101`
- Batch header (optional): `🔧 Executing 3 tools…`

Authoring guidance for docs and examples:
- Prefer short targets (basename for files, truncated args for commands)
- Keep details (full JSON, long outputs) out of main flow; link or reference
- Use the same symbols and ordering for consistency across docs and UI

When updating screenshots or walkthroughs, match the above style so users see the same patterns in the TUI.

## TUI Interaction Conventions

For consistency across docs and the app, use and document these keybindings:
- Enter: submit; Shift/Ctrl+Enter: newline
- Tab: accept autocomplete (when visible); Esc: close autocomplete
- Ctrl+M: open model selection; `/model` also supported
- Shift+Tab: cycle modes (default/plan/auto-accept/turbo)

Planned configuration:
- Document `ui.submit_on_enter` if/when added, and indicate how newline shortcuts can be customized.

## Progress & Status Conventions

Keep progress information consistent and scannable across the project. Use the following structure and locations.

- STATUS.md (authoritative, high level)
  - Last Updated (ISO date)
  - Recent Fixes (bulleted, concise, user-visible)
  - Current Focus (phase, e.g., Phase 2 Tool Loop)
  - Recent Progress (what landed this week)
  - Next Steps (3–5 bullets with clear outcomes)
  - Known Issues (prioritized)

- TODO.md (actionable work)
  - Immediate tasks (this week)
  - Next sprint/backlog
  - Recently completed (rolling list with dates)

- AGENTS.md (AI-facing quick context)
  - “What works today” vs. “Current priority”
  - Keyboard shortcuts and interaction patterns (kept up-to-date)
  - Short notes for streaming/compaction behavior

- TECH_SPEC.md (technical truth)
  - Current state of the agent loop (connected/streaming/tool status)
  - Keybindings + config excerpts (e.g., `ui.submit_on_enter`)
  - Pointers to roadmap and status

- Roadmap (docs/architecture/roadmap.md)
  - Phase summaries (progress + next steps)
  - Success criteria per phase
  - Risks and mitigations

Update triggers:
- Behavior changes that affect users or test flows (e.g., keybindings, streaming placement, timeouts)
- Provider integration changes (model defaults/fallbacks, error surfaces)
- Tool loop architecture changes (parsing, streaming, iteration rules)

Checklist for updates:
- [ ] STATUS.md (date, recent fixes, next steps)
- [ ] TODO.md (immediate + recently completed)
- [ ] AGENTS.md (shortcuts, streaming/compaction notes)
- [ ] TECH_SPEC.md (current state, config)
- [ ] Roadmap (progress + next tasks)

Style guidelines:
- Be concise and specific (1–2 lines per bullet)
- Use ISO dates (YYYY-MM-DD)
- Prefer imperative mood for next steps (e.g., “Add collapsible results”)
- Link to code paths when useful (e.g., `src/ui/mod.rs`)

MVP readiness rubric:
- [ ] Non-blocking send/receive
- [ ] Adaptive timeouts and clear errors
- [ ] Predictive compaction and context safety
- [ ] Provider/model preflight (overlay when missing)
- [ ] Tool loop iteration cap + duplicate-batch guard
- [ ] Truncated tool results + readable status lines

## Predictive Compaction

To prevent context-limit errors mid-turn, the TUI predicts when a new message may exceed ~85% of the model’s context window and triggers compaction (if enabled):
- Auto-enabled: compact immediately, with a brief system notice.
- Auto-disabled with warnings: show a warning suggesting `/compact`.
- Estimation is lightweight (chars/4 + fixed overhead) and model-aware via provider context window.

When documenting flows or creating screenshots, prefer sequences that show the system’s proactive compaction note when approaching limits.

## Quality Checklist

Before committing documentation changes:

- [ ] No duplicate information across files?
- [ ] Updated REFERENCE.md if moved content?
- [ ] Used ISO dates (2025-08-23)?
- [ ] Archived old versions if major change?
- [ ] Updated AGENTS.md if significant?
- [ ] Verified cross-references still work?
- [ ] Removed any outdated information?
- [ ] Added to DECISIONS.md if notable?

## Common Mistakes to Avoid

1. **Creating new files for existing info** - Check REFERENCE.md first
2. **Duplicating facts** - Each fact in ONE place only
3. **Not updating references** - Break links when moving content
4. **Keeping contradictions** - Archive old, keep current
5. **Wrong date format** - Use YYYY-MM-DD not MM/DD/YY
6. **Forgetting DECISIONS.md** - Document WHY not just WHAT
7. **Bloating AGENTS.md** - Keep it minimal, link to details

---
*General-purpose documentation guide for any project.*
