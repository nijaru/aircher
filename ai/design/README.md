# Implementation Design Specs

**Purpose**: API surface and integration points (no verbose code)

## When to Use design/

**Use design/ for:**
- API signatures and class interfaces
- Integration points between systems
- Weekly implementation checklists
- Data flow diagrams (if needed)

**Don't use design/ for:**
- ❌ Full code implementations (belongs in src/)
- ❌ Verbose examples (research has patterns)
- ❌ Duplicate schemas (research has SQL/algorithms)

## Current Specs

**week1-api-surface.md** (82 lines) - Classes to implement, schema references, daily checklist

## Pattern

Each spec should be **<100 lines**:
1. **Class names and methods** (signatures only)
2. **Key decisions** (which library, why)
3. **Integration points** (how it connects)
4. **Daily checklist** (deliverables)
5. **References** (research file line numbers)

**Example good spec**:
```markdown
## DuckDB Memory
**File**: src/aircher/memory/duckdb_memory.py
**Schema**: Lines 85-166 in memory-system-architecture.md

class DuckDBMemory:
    record_tool_execution(...)
    get_file_history(...) -> List[Dict]

Day 1: Create schema
Day 2: Implement recording
```

**Example bad spec**:
```python
# 500 lines of full implementation code
class DuckDBMemory:
    def __init__(self):
        # Full implementation here...
```

---

**Keep it concise. Research has the details.**
