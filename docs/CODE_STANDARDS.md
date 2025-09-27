# Code Standards & AI Agent Guidelines
*Quick Reference for Development & Documentation*

> **📚 Primary Reference**: See `external/agent-contexts/standards/AI_CODE_PATTERNS.md` for comprehensive coding patterns and decision trees.

## Quick Start for AI Agents

1. **Check Context**: Read project AGENTS.md if exists
2. **Before Any Code**: Check existing patterns with `rg`
3. **Core Rule**: Edit existing files, never create without explicit request
4. **Test Everything**: Performance claims need benchmarks
5. **Clean Up**: Delete all test files before finishing

## Aircher-Specific Guidelines

This file contains project-specific guidelines. For universal patterns, see:
- `external/agent-contexts/standards/AI_CODE_PATTERNS.md` - Universal coding patterns
- `external/agent-contexts/AI_AGENT_INDEX.md` - Navigation and decision trees

## Priority Levels
- **MUST** = Always required, no exceptions
- **SHOULD** = Default behavior unless justified
- **MAY** = Optional based on context

## Core Principles (Apply to Everything)

1. **Clarity > Cleverness** - Names must reveal intent
2. **Scope = Length** - Short scope → short name, broad scope → descriptive
3. **Self-Contained** - Make sense without context, avoid redundancy
4. **Precise Units** - Include units in names: `timeoutMs`, `bufferKB`
5. **Consistency** - Follow project patterns exactly

## Quick Reference Tables

### Variables & Constants

| Type | Pattern | Examples |
|------|---------|----------|
| Local Variables | Short, clear | `i`, `n`, `err`, `result` |
| Shared Variables | Descriptive | `activeSessionCount`, `retryLimit` |
| Booleans | Question form | `isEnabled`, `hasPermission`, `canRetry` |
| Numbers | Include units | `timeoutMs`, `ratePerSecond`, `maxRetries` |
| Constants | UPPER_SNAKE or project style | `MAX_RETRIES`, `DEFAULT_TIMEOUT_MS` |
| Collections | Plural | `users`, `items`, `connections` |
| Single Element | Singular | `user`, `item`, `connection` |

### Functions & Methods

| Purpose | Pattern | Examples |
|---------|---------|----------|
| Actions | Verb phrases | `parseFile()`, `calculateChecksum()` |
| Queries | Get/Find/Check | `getUser()`, `findById()`, `checkStatus()` |
| Commands | Action verbs | `saveUser()`, `deleteFile()`, `resetCache()` |
| Async | Language convention | `fetchAsync()`, `await upload()` |
| Side Effects | Clear mutation | `updateDatabase()` not `getData()` |
| Symmetry | Paired operations | `open/close`, `start/stop`, `push/pop` |

### Types & Classes

| Type | Pattern | Examples |
|------|---------|----------|
| Classes | Noun phrases | `PaymentRequest`, `ImageDecoder` |
| Interfaces | Descriptive nouns | `Cacheable`, `Serializer` |
| Enums | Singular + noun members | `Status.Pending`, `Level.Warning` |
| Avoid | Generic terms | ❌ `Data`, `Info`, `Manager`, `Helper` |

### Files & Modules

| Language | File Pattern | Test Pattern |
|----------|-------------|--------------|
| Python | `snake_case.py` | `test_snake_case.py` |
| JavaScript/TypeScript | `kebab-case.ts` | `kebab-case.test.ts` |
| Go | `lowercase.go` | `lowercase_test.go` |
| Mojo | `snake_case.mojo` | `test_snake_case.mojo` |

## Documentation Rules

### Comments - The "Why" Not "What"

```python
# BAD: Increment counter
counter += 1

# GOOD: Exponential backoff to protect downstream service
delay_ms = base_delay * (2 ** retry_count)
```

### Function Documentation Template

```python
def process_batch(vectors: np.ndarray, batch_size: int = 1000) -> List[str]:
    """Process vectors in batches, returning IDs.
    
    Args:
        vectors: Input vectors as float32 array (N x D)
        batch_size: Vectors per batch (default: 1000)
    
    Returns:
        List of generated IDs
        
    Raises:
        ValueError: If vectors shape invalid
        
    Performance:
        ~85K vectors/second on M1 Mac
    """
```

### Error Messages - Actionable

```python
# BAD
raise Error("Failed")

# GOOD  
raise ValueError(f"Vector dimension {dim} doesn't match index dimension {expected_dim}")
```

## Language-Specific Conventions

### Python
- Files: `snake_case.py`
- Classes: `PascalCase`
- Functions/variables: `snake_case`
- Constants: `UPPER_SNAKE_CASE`
- Private: `_leading_underscore`
- Tests: `test_*.py` or `*_test.py`

### JavaScript/TypeScript
- Files: `kebab-case.ts` or `camelCase.ts`
- Classes: `PascalCase`
- Functions/variables: `camelCase`
- Constants: `UPPER_SNAKE_CASE`
- Private: `#privateMember` or `_convention`
- Tests: `*.test.ts` or `*.spec.ts`

### Go
- Files: `lowercase.go` (no underscores)
- Exported: `CapitalCase`
- Unexported: `lowerCase`
- Tests: `*_test.go`

### Performance Testing Requirements
- [MUST] Test with production data sizes
- [MUST] Include units: `req_per_sec`, `latency_ms`
- [MUST] Be precise: `156,937` not `~157K`
- [MUST] Document conditions: `# Hardware, data size, version`
- [SHOULD] Run benchmarks 3x, report median

### API Design Principles

```python
# Batch operations for performance
process_batch(items)        # Good: Single operation
for item in items: process(item)  # Bad: Multiple calls

# Memory efficiency
array.astype(np.float32)    # Good: Efficient type
array.tolist()              # Bad: Unnecessary copy

# API stability (backward compatibility)
def process(data):          # Never change signature
def process_v2(data, **):   # Add new method if needed
```

## Commit Message Format

```
type(scope): imperative description

- What changed and why
- Performance impact if relevant
- Breaking changes marked with BREAKING

Fixes #123
```

Examples:
```
feat(diskann): add batch insertion support
fix(ffi): remove memory leak in vector transfer  
perf(search): optimize SIMD distance calculation
docs(api): update batch_add documentation
```

## AI Agent Working Rules

### Decision Tree (Start Here)
```
Task requires multiple files?
├─ NO → Edit existing file (MUST)
└─ YES → User explicitly asked for new file?
    ├─ NO → Find way to use existing files (MUST)
    └─ YES → Create with clear purpose

Performance claim needed?
├─ NO → Continue
└─ YES → Run actual benchmark 3x (MUST)
    └─ Document exact conditions

Code doesn't work as expected?
├─ Is it blocking the task?
│  ├─ YES → Fix it (MUST)
│  └─ NO → Note in comment, continue
└─ Would refactoring help?
   ├─ User asked for it → Refactor
   └─ User didn't ask → Leave as-is (MUST)
```

### Before Starting
- [MUST] Check existing patterns in codebase first
- [MUST] Read relevant files before creating new ones
- [MUST] Use `rg` to find similar implementations
- [SHOULD] Check if functionality already exists
- [MUST] Preserve all existing functionality

### During Implementation
- [MUST] Edit existing files over creating new ones
- [MUST] Never create test/temp files without cleanup
- [MUST] Match exact indentation (tabs vs spaces)
- [MUST] Follow import order from existing files
- [SHOULD] Use batch operations over loops
- [MUST] Test claims before documenting them
- [MUST] Keep changes minimal and focused
- [SHOULD] Avoid "drive-by" improvements

### Clean Up Rules
- [MUST] Delete all temporary test files (`test.py`, `temp.mojo`, etc.)
- [MUST] Remove debug print statements
- [MUST] Clean up commented-out code
- [SHOULD] Delete unused imports
- [MUST] Remove `.pyc`, `__pycache__`, `.DS_Store`
- [MUST] Never commit API keys or test credentials
- [MUST] Run `rm test_*.py temp_*.mojo` before finishing

### File Creation Guidelines
```bash
# NEVER create these without explicit request:
README.md          # Only if user asks
CONTRIBUTING.md    # Only if user asks  
test_*.py         # Clean up after testing
temp_*.mojo       # Delete when done
debug_*.log       # Remove before finishing
*.tmp, *.bak      # Always clean up
```

### Testing Patterns
```python
# Create temporary test file
echo "test_vector_perf.py"  # Name indicates temporary

# Run test
python test_vector_perf.py

# ALWAYS delete after
rm test_vector_perf.py
```

### Performance Testing Rules
- [ ] Run performance tests 3+ times
- [ ] Document exact conditions (hardware, data size)
- [ ] Use production data shapes (not toy examples)
- [ ] Clean up large test data files
- [ ] Never claim untested performance

## AI Agent Checklist

When reviewing/writing code, verify:

- [ ] Names reveal intent without comments
- [ ] Units included in numeric variables
- [ ] Booleans read as questions
- [ ] Functions are verbs describing action
- [ ] No generic terms (Data, Info, Manager)
- [ ] Comments explain "why" not "what"
- [ ] Error messages are actionable
- [ ] Test names describe behavior
- [ ] File organization follows patterns
- [ ] All temp files cleaned up
- [ ] No debug code remains

## Common AI Agent Mistakes (Avoid These)

### Over-Engineering
```python
# BAD: Creating abstraction for single use
class VectorProcessorFactoryBuilder: ...

# GOOD: Direct implementation
def process_vectors(vectors): ...
```

### Aggressive Refactoring
```python
# BAD: "Improving" working code without being asked
# Original: if x > 0: return True else: return False
# Changed to: return x > 0  # User didn't ask for this

# GOOD: Leave working code alone unless task requires change
```

### Creating Unnecessary Files
```bash
# BAD: Creating files "for organization"
utils/helpers.py       # Generic name
common/shared.py       # Vague purpose
test_everything.py     # Not cleaned up

# GOOD: Edit existing files, create only when essential
```

### Untested Performance Claims
```python
# BAD: "This improves performance by 50%"  # Never tested

# GOOD: Run benchmark, document results
# Benchmark: 156,937 vec/s (M1 Mac, 128D, float32)
```

## Version Control Safety

### Git Rules
- [MUST] Never force push without permission
- [MUST] Never rebase main/master branches  
- [MUST] Never commit sensitive data
- [SHOULD] Commit message < 72 chars first line
- [MAY] Use conventional commits (feat/fix/docs)

### Before Committing
```bash
# MUST run these checks:
git status          # Check what's being committed
git diff --staged   # Review actual changes
rg "TODO|FIXME"     # Find incomplete work
ls test_*.py        # Find temp files to clean
```

## Quick Fixes

| Bad | Good | Why |
|-----|------|-----|
| `tmp` | `configPath` | Reveals purpose |
| `process()` | `processPayment()` | Specific action |
| `data` | `userProfile` | Domain-specific |
| `cfg` | `config` | No abbreviations |
| `// increment i` | `// Retry with backoff` | Explain why |
| `Error: failed` | `Error: timeout after 30s` | Actionable |
| `test1()` | `test_handles_empty_input()` | Describes behavior |
| Creating new file | Editing existing | Maintains structure |
| "~50% faster" | "47,892 vec/s" | Precise measurement |
| Refactoring everything | Focused changes | Respects existing code |

## When to Ask User for Clarification

- [MUST] Ask when multiple valid interpretations exist
- [MUST] Ask before breaking changes
- [SHOULD] Ask before major refactoring
- [SHOULD] Ask when performance tradeoffs needed
- [MAY] Ask about code style preferences

## Final Checklist Before Completing Task

- [ ] All tests pass
- [ ] No temp files remain
- [ ] No debug prints left
- [ ] Performance claims tested
- [ ] Documentation updated if needed
- [ ] No unnecessary files created
- [ ] Existing functionality preserved
- [ ] Changes match user's request exactly

## TUI Tool Line Guidelines

When emitting tool status/result messages in the TUI:

- Use compact, single‑line updates with consistent symbols and order.
  - Running: `🔧 <tool> <target> — running…`
  - Success: `✓ <tool> <target> — <summary>`
  - Error: `✗ <tool> <target> — <reason>`
- Prefer short targets:
  - Files: show basename or a shortened `…/file.rs` form
  - Commands: truncate args to ~60 chars with an ellipsis
- Group multiple tools with a batch header: `🔧 Executing N tools…`
- Keep JSON/details out of these lines; summaries only.
- Color comes from message role (Tool), not inline ANSI.

Rationale: This mirrors successful patterns in Claude Code/Codex/Gemini and optimizes scan‑ability in a terminal UI.

## TUI Keybinding Guidelines

- Enter defaults to submit; Shift/Ctrl+Enter insert newline.
- Tab accepts autocomplete when visible; otherwise insert four spaces (indent).
- Esc closes autocomplete or interrupts streaming.
- Provide an accessible model switch: Ctrl+M and `/model`.
- Avoid overloading Tab with modal actions; keep behavior predictable.

Planned: If adding configurable Enter behavior, prefer a single boolean `ui.submit_on_enter` and a small set of allowed newline shortcuts to prevent fragmentation.

---
*This is a general-purpose code standards guide. For project-specific patterns, check the project's AGENTS.md or README.*
