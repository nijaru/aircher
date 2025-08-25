# Documentation Standards & Organization Guide
*General-purpose guide for project documentation*

## Core Principles

### 1. Single Source of Truth
Every piece of information lives in EXACTLY ONE place. Never duplicate facts across files.

### 2. Clear Hierarchy
```
README.md           # Public introduction
CLAUDE.md           # AI agent context (if using AI)
STATUS.md           # Current state (what is)
TECH_SPEC.md        # Architecture (how it works)
TODO.md             # Tasks (what needs doing)
DECISIONS.md        # History (why decisions were made)
```

### 3. Token Efficiency
- Reference, don't duplicate: `See STATUS.md#performance`
- Use anchors for sections: `#known-issues`
- Keep summaries in CLAUDE.md minimal
- Full details in dedicated files

## File Organization

### What Goes Where

| Information Type | Location | Example |
|-----------------|----------|---------|
| Quick context | CLAUDE.md | "Project overview for AI agents" |
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
4. Update CLAUDE.md if major milestone

### For Bug Fixes
1. Fix the bug
2. Update STATUS.md#known-issues
3. Remove from TODO.md
4. Document solution in DECISIONS.md if notable

### For Architecture Changes
1. Document decision in DECISIONS.md FIRST
2. Update TECH_SPEC.md with details
3. Update STATUS.md if impacts current state
4. Update CLAUDE.md summary if major

## AI Agent Optimization

### File Inclusion Strategy

**Minimal context (fast responses):**
```bash
@CLAUDE.md  # Always, provides base context
```

**Standard work:**
```bash
@CLAUDE.md
@docs/STATUS.md  # Add current state
```

**Complex work:**
```bash
@CLAUDE.md
@docs/STATUS.md
@docs/TECH_SPEC.md  # Add technical details
@docs/TODO.md       # Add task context
```

### Token-Saving Tips

1. **Use section anchors**: `@docs/STATUS.md#performance-metrics`
2. **Reference don't copy**: "See TECH_SPEC.md for details"
3. **Summarize in CLAUDE.md**: Keep details in dedicated files
4. **Archive old content**: Don't keep outdated info in active files

## Quality Checklist

Before committing documentation changes:

- [ ] No duplicate information across files?
- [ ] Updated REFERENCE.md if moved content?
- [ ] Used ISO dates (2025-08-23)?
- [ ] Archived old versions if major change?
- [ ] Updated CLAUDE.md if significant?
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
7. **Bloating CLAUDE.md** - Keep it minimal, link to details

---
*General-purpose documentation guide for any project.*