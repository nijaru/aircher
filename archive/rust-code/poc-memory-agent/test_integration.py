#!/usr/bin/env python3
"""
Integration test: Knowledge Graph + Episodic Memory

Demonstrates how memory-enhanced agent would work:
1. Build knowledge graph from codebase
2. Query graph for relevant code structure
3. Record actions to episodic memory
4. Retrieve historical context
5. Combine for enhanced decision making
"""

import time
from pathlib import Path

from episodic_memory import EpisodicMemory
from knowledge_graph import CodeGraphBuilder


def test_memory_enhanced_workflow():
    """
    Simulate an agent working on a bug fix with memory enhancement.

    Scenario: "Fix authentication bug"

    Without memory:
    - Agent reads many files blindly
    - No knowledge of code structure
    - Repeats past mistakes

    With memory:
    - Graph shows auth.rs contains login() function
    - Episodic memory shows "I've worked on auth.rs before"
    - Patterns show "auth bugs usually involve middleware.rs too"
    - Agent makes smarter decisions
    """

    print("=" * 60)
    print("Memory-Enhanced Agent Workflow Test")
    print("=" * 60)

    # Initialize systems
    print("\n[1] Initializing knowledge graph and episodic memory...")
    graph = CodeGraphBuilder()
    memory = EpisodicMemory(Path("test_workflow.db"))

    # Build knowledge graph (do this once, reuse later)
    print("\n[2] Building knowledge graph from codebase...")
    repo_root = Path("../src")

    if repo_root.exists():
        start = time.time()
        stats = graph.scan_repository(repo_root)
        elapsed = time.time() - start

        print(f"    Scanned {stats['files_scanned']} files in {elapsed:.2f}s")
        print(
            f"    Extracted {stats['total_nodes']} nodes, {stats['total_edges']} edges"
        )
        print(f"    Node types: {stats['node_types']}")
    else:
        print(f"    Warning: {repo_root} not found, using mock data")

    # Simulate agent workflow
    print("\n[3] Agent receives task: 'Fix authentication bug'")
    task_context = "Fix authentication bug"

    # Step 1: Query knowledge graph
    print("\n[4] Query knowledge graph for relevant code:")
    auth_file = "../src/agent/intelligence/mod.rs"  # Adjust to real file

    print("\n    Graph query: What's in the auth module?")
    # This would return actual functions if auth.rs existed
    # For demo, we'll show the concept

    print("    → Functions: login(), validate_token(), refresh_session()")
    print("    → Types: User, Session, AuthError")

    # Step 2: Query episodic memory
    print("\n[5] Query episodic memory for historical context:")

    # Check if we've worked on this file before
    history = memory.get_file_history(auth_file, limit=5)

    if history:
        print(f"    → Found {len(history)} past interactions")
        for ep in history:
            print(f"      • {ep.tool} - {ep.context}")
    else:
        print("    → No previous history (first time working on this file)")
        # Record this as first interaction
        memory.record_action(
            tool="read_file",
            file_path=auth_file,
            success=True,
            duration_ms=150,
            context=task_context,
        )

    # Step 3: Check learned patterns
    print("\n[6] Query learned patterns:")

    patterns = memory.get_patterns(pattern_type="co_edit")
    if patterns:
        print(f"    → Found {len(patterns)} co-edit patterns")
        for p in patterns:
            print(f"      • {p.description}")
            print(f"        Files: {p.files}")
            print(f"        Confidence: {p.confidence}")
    else:
        print("    → No patterns learned yet")

    # Step 4: Query graph for dependencies
    print("\n[7] Find related files via graph:")

    # This would use actual graph queries
    # For demo, we show the concept
    print("    Graph query: What does auth module depend on?")
    print("    → Calls: hash_password(), validate_email()")
    print("    → Uses: Database, Config")

    # Step 5: Make decision with enhanced context
    print("\n[8] Agent decision with memory enhancement:")

    print("\n    WITHOUT memory:")
    print("      • Agent would read 10+ files sequentially")
    print("      • No knowledge of code structure")
    print("      • Might repeat past failed approaches")
    print("      • Tool calls: ~15-20")

    print("\n    WITH memory:")
    print("      • Graph shows login() is the relevant function")
    print("      • History shows recent work on validation logic")
    print("      • Pattern suggests checking middleware too")
    print("      • Tool calls: ~5-8 (40-60% reduction!)")

    # Step 6: Record the workflow
    print("\n[9] Recording this workflow to memory...")

    # Simulate sequence of actions
    actions = [
        ("read_file", auth_file, "Reading auth module"),
        ("read_file", "../src/middleware/auth.rs", "Checking middleware"),
        ("edit_file", auth_file, "Fixed validation bug"),
    ]

    for tool, file, ctx in actions:
        memory.record_action(
            tool=tool, file_path=file, success=True, context=task_context
        )
        print(f"    Recorded: {tool} on {Path(file).name}")

    # Step 7: Learn pattern for future
    print("\n[10] Learning pattern from this workflow:")

    # Detect co-edited files
    co_edited = [a[1] for a in actions if a[0] in ("edit_file", "write_file")]

    if len(co_edited) >= 2:
        memory.learn_pattern(
            pattern_type="co_edit",
            description=f"When working on '{task_context}', these files are related",
            files=co_edited,
            confidence=0.75,
        )
        print(f"    Learned: {len(co_edited)} files typically edited together")

    # Summary
    print("\n" + "=" * 60)
    print("SUMMARY: Memory Enhancement Benefits")
    print("=" * 60)

    stats = memory.get_stats()
    print("\nMemory Statistics:")
    print(f"  Total actions recorded: {stats['total_episodes']}")
    print(f"  Success rate: {stats['success_rate']:.1%}")
    print(f"  Learned patterns: {stats['learned_patterns']}")
    print(f"  Files touched: {stats['files_touched']}")

    print("\nExpected Performance Improvement:")
    print("  ✓ 40-60% fewer tool calls (graph-guided navigation)")
    print("  ✓ 30-50% fewer files examined (history-informed)")
    print("  ✓ Higher success rate (learned patterns applied)")

    print("\nNext: Run benchmark.py to validate these claims!")

    memory.close()


if __name__ == "__main__":
    test_memory_enhanced_workflow()
