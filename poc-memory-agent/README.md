# Memory-Enhanced Agent POC

**Goal**: Validate that knowledge graph + episodic memory improves coding agent performance

## Research Question

"Does persistent memory (code structure + action history) reduce tool calls and improve task success rates compared to stateless agents?"

## Architecture

```
┌─────────────────────────────────────┐
│  Memory-Enhanced Agent              │
├─────────────────────────────────────┤
│  1. Knowledge Graph                 │
│     - Extract code structure        │
│     - Store: files, classes, funcs  │
│     - Edges: contains, calls, uses  │
│                                     │
│  2. Episodic Memory                 │
│     - Track all actions             │
│     - Learn co-edit patterns        │
│     - Retrieve relevant history     │
│                                     │
│  3. Query Interface                 │
│     - "What's in this file?"        │
│     - "What have I done here?"      │
│     - "What patterns apply?"        │
└─────────────────────────────────────┘
```

## Components

### knowledge_graph.py
- Extract AST with tree-sitter (Rust support)
- Build NetworkX graph: nodes (files, classes, functions), edges (relationships)
- Query interface for code structure questions

### episodic_memory.py
- SQLite database for action tracking
- Record: tool calls, files touched, success/failure
- Query: history, co-edit patterns, learned insights

### benchmark.py
- Compare agent with/without memory
- Metrics: tool calls, files examined, task success
- Test tasks: bug fixing, feature addition, refactoring

## Setup

```bash
pip install tree-sitter tree-sitter-rust networkx
python knowledge_graph.py  # Build graph from ../src
python benchmark.py        # Run tests
```

## Hypothesis

**With memory**, agent should:
- Examine 30-50% fewer files (graph guides to relevant code)
- Make 20-40% fewer tool calls (history informs decisions)
- Have higher task success rate (patterns learned from past work)

## Timeline

- **Week 1**: Build knowledge graph + episodic memory
- **Week 2**: Benchmark, validate, document findings

## Next Steps After POC

If results show >25% improvement:
1. Port to Rust (integrate with existing Aircher agent)
2. Wire up ACP protocol
3. Blog post series + potential paper
