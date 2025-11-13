"""Unit tests for NetworkX-based knowledge graph."""

import pickle
from pathlib import Path

import pytest

from aircher.memory.knowledge_graph import KnowledgeGraph


@pytest.fixture
def graph():
    """Create a fresh knowledge graph instance."""
    return KnowledgeGraph()


@pytest.fixture
def populated_graph():
    """Create a graph with some test data."""
    kg = KnowledgeGraph()

    # Add a file
    file_id = kg.add_file("/src/main.py", "python")

    # Add some functions
    kg.add_function(
        name="main",
        signature="main()",
        line=1,
        file_path="/src/main.py",
        file_node_id=file_id,
    )

    kg.add_function(
        name="helper",
        signature="helper(x, y)",
        line=10,
        file_path="/src/main.py",
        file_node_id=file_id,
    )

    # Add a class
    kg.add_class(
        name="Calculator",
        line=20,
        file_path="/src/main.py",
        file_node_id=file_id,
        methods=["add", "subtract"],
    )

    return kg


class TestKnowledgeGraphInitialization:
    """Test knowledge graph initialization."""

    def test_empty_graph_creation(self):
        """Test creating an empty knowledge graph."""
        kg = KnowledgeGraph()
        assert kg.graph is not None
        assert kg.extractor is not None
        assert kg.graph.number_of_nodes() == 0
        assert kg.graph.number_of_edges() == 0

    def test_graph_is_directed(self, graph):
        """Test that graph is a directed graph."""
        import networkx as nx

        assert isinstance(graph.graph, nx.DiGraph)


class TestFileNodes:
    """Test file node operations."""

    def test_add_file(self, graph):
        """Test adding a file node."""
        file_id = graph.add_file("/src/main.py", "python")

        assert file_id == "file:/src/main.py"
        assert graph.graph.has_node(file_id)

        node_data = graph.graph.nodes[file_id]
        assert node_data["type"] == "file"
        assert node_data["path"] == "/src/main.py"
        assert node_data["language"] == "python"

    def test_add_multiple_files(self, graph):
        """Test adding multiple files."""
        file1_id = graph.add_file("/src/main.py", "python")
        file2_id = graph.add_file("/src/utils.rs", "rust")

        assert graph.graph.number_of_nodes() == 2
        assert file1_id != file2_id


class TestFunctionNodes:
    """Test function node operations."""

    def test_add_function(self, graph):
        """Test adding a function node."""
        file_id = graph.add_file("/src/main.py", "python")

        func_id = graph.add_function(
            name="test_func",
            signature="test_func(a, b)",
            line=10,
            file_path="/src/main.py",
            file_node_id=file_id,
        )

        assert func_id == "function:/src/main.py:test_func:10"
        assert graph.graph.has_node(func_id)

        node_data = graph.graph.nodes[func_id]
        assert node_data["type"] == "function"
        assert node_data["name"] == "test_func"
        assert node_data["signature"] == "test_func(a, b)"
        assert node_data["line"] == 10

    def test_function_contains_edge(self, graph):
        """Test that contains edge is created from file to function."""
        file_id = graph.add_file("/src/main.py", "python")
        func_id = graph.add_function(
            name="test",
            signature="test()",
            line=1,
            file_path="/src/main.py",
            file_node_id=file_id,
        )

        assert graph.graph.has_edge(file_id, func_id)
        edge_data = graph.graph.edges[file_id, func_id]
        assert edge_data["type"] == "contains"

    def test_add_multiple_functions(self, graph):
        """Test adding multiple functions to same file."""
        file_id = graph.add_file("/src/main.py", "python")

        func1_id = graph.add_function(
            name="func1", signature="func1()", line=1, file_path="/src/main.py", file_node_id=file_id
        )

        func2_id = graph.add_function(
            name="func2", signature="func2()", line=10, file_path="/src/main.py", file_node_id=file_id
        )

        # Both functions should be connected to the file
        assert graph.graph.has_edge(file_id, func1_id)
        assert graph.graph.has_edge(file_id, func2_id)


class TestClassNodes:
    """Test class node operations."""

    def test_add_class(self, graph):
        """Test adding a class node."""
        file_id = graph.add_file("/src/main.py", "python")

        class_id = graph.add_class(
            name="MyClass",
            line=5,
            file_path="/src/main.py",
            file_node_id=file_id,
            methods=["method1", "method2"],
        )

        assert class_id == "class:/src/main.py:MyClass:5"
        assert graph.graph.has_node(class_id)

        node_data = graph.graph.nodes[class_id]
        assert node_data["type"] == "class"
        assert node_data["name"] == "MyClass"
        assert node_data["line"] == 5
        assert "method1" in node_data["methods"]

    def test_class_contains_edge(self, graph):
        """Test that contains edge is created from file to class."""
        file_id = graph.add_file("/src/main.py", "python")
        class_id = graph.add_class(
            name="Test",
            line=1,
            file_path="/src/main.py",
            file_node_id=file_id,
        )

        assert graph.graph.has_edge(file_id, class_id)
        edge_data = graph.graph.edges[file_id, class_id]
        assert edge_data["type"] == "contains"

    def test_class_without_methods(self, graph):
        """Test adding a class without methods."""
        file_id = graph.add_file("/src/main.py", "python")
        class_id = graph.add_class(
            name="Empty",
            line=1,
            file_path="/src/main.py",
            file_node_id=file_id,
        )

        node_data = graph.graph.nodes[class_id]
        assert node_data["methods"] == []


class TestImportNodes:
    """Test import node operations."""

    def test_add_import(self, graph):
        """Test adding an import node."""
        file_id = graph.add_file("/src/main.py", "python")

        import_id = graph.add_import(
            file_node_id=file_id,
            module="os",
            items=["path", "environ"],
        )

        assert import_id == f"import:{file_id}:os"
        assert graph.graph.has_node(import_id)

        node_data = graph.graph.nodes[import_id]
        assert node_data["type"] == "import"
        assert node_data["module"] == "os"
        assert "path" in node_data["items"]

    def test_import_edge(self, graph):
        """Test that imports edge is created."""
        file_id = graph.add_file("/src/main.py", "python")
        import_id = graph.add_import(
            file_node_id=file_id,
            module="sys",
            items=[],
        )

        assert graph.graph.has_edge(file_id, import_id)
        edge_data = graph.graph.edges[file_id, import_id]
        assert edge_data["type"] == "imports"

    def test_full_module_import(self, graph):
        """Test importing full module without specific items."""
        file_id = graph.add_file("/src/main.py", "python")
        import_id = graph.add_import(
            file_node_id=file_id,
            module="json",
            items=[],
        )

        node_data = graph.graph.nodes[import_id]
        assert node_data["items"] == []


class TestCallEdges:
    """Test function call edges."""

    def test_add_call_edge(self, graph):
        """Test adding a call edge between functions."""
        file_id = graph.add_file("/src/main.py", "python")

        caller_id = graph.add_function(
            name="caller", signature="caller()", line=1, file_path="/src/main.py", file_node_id=file_id
        )

        callee_id = graph.add_function(
            name="callee", signature="callee()", line=5, file_path="/src/main.py", file_node_id=file_id
        )

        graph.add_call_edge(caller_id, callee_id)

        assert graph.graph.has_edge(caller_id, callee_id)
        edge_data = graph.graph.edges[caller_id, callee_id]
        assert edge_data["type"] == "calls"


class TestBuildFromFile:
    """Test building graph from source files."""

    def test_build_from_python_file(self, tmp_path, graph):
        """Test building graph from a Python file."""
        # Create a test Python file
        test_file = tmp_path / "test.py"
        test_file.write_text("""
def function1():
    pass

def function2(x, y):
    return x + y

class MyClass:
    def method1(self):
        pass
""")

        file_node_id = graph.build_from_file(test_file, "python")

        # Verify file node
        assert graph.graph.has_node(file_node_id)
        assert graph.graph.nodes[file_node_id]["type"] == "file"

        # Verify functions were extracted
        file_contents = graph.get_file_contents(str(test_file))
        assert len(file_contents["functions"]) >= 2
        assert len(file_contents["classes"]) >= 1

        # Check function names
        func_names = [f["name"] for f in file_contents["functions"]]
        assert "function1" in func_names
        assert "function2" in func_names

    def test_build_from_rust_file(self, tmp_path, graph):
        """Test building graph from a Rust file."""
        test_file = tmp_path / "test.rs"
        test_file.write_text("""
fn main() {
    println!("Hello");
}

fn helper(x: i32) -> i32 {
    x + 1
}

struct MyStruct {
    field: i32,
}
""")

        file_node_id = graph.build_from_file(test_file, "rust")

        # Verify file node
        assert graph.graph.has_node(file_node_id)

        # Verify functions were extracted
        file_contents = graph.get_file_contents(str(test_file))
        func_names = [f["name"] for f in file_contents["functions"]]
        assert "main" in func_names or "helper" in func_names

    def test_build_from_file_with_imports(self, tmp_path, graph):
        """Test extracting imports from file."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
import os
from pathlib import Path
""")

        file_node_id = graph.build_from_file(test_file, "python")

        file_contents = graph.get_file_contents(str(test_file))
        assert len(file_contents["imports"]) > 0


class TestGraphQueries:
    """Test querying the graph."""

    def test_get_file_contents(self, populated_graph):
        """Test getting file contents."""
        contents = populated_graph.get_file_contents("/src/main.py")

        assert len(contents["functions"]) == 2
        assert len(contents["classes"]) == 1

        # Check function details
        func_names = [f["name"] for f in contents["functions"]]
        assert "main" in func_names
        assert "helper" in func_names

        # Check class details
        assert contents["classes"][0]["name"] == "Calculator"

    def test_get_file_contents_nonexistent(self, graph):
        """Test getting contents of nonexistent file."""
        contents = graph.get_file_contents("/nonexistent.py")

        assert contents["functions"] == []
        assert contents["classes"] == []
        assert contents["imports"] == []

    def test_find_symbol(self, populated_graph):
        """Test finding symbols by name."""
        results = populated_graph.find_symbol("main")

        assert len(results) == 1
        assert results[0]["type"] == "function"
        assert results[0]["name"] == "main"
        assert results[0]["file_path"] == "/src/main.py"
        assert results[0]["line"] == 1

    def test_find_symbol_no_matches(self, populated_graph):
        """Test finding symbol with no matches."""
        results = populated_graph.find_symbol("nonexistent_symbol")

        assert len(results) == 0

    def test_find_symbol_multiple_matches(self, graph):
        """Test finding symbol with multiple occurrences."""
        file1_id = graph.add_file("/src/file1.py", "python")
        file2_id = graph.add_file("/src/file2.py", "python")

        graph.add_function("test", "test()", 1, "/src/file1.py", file1_id)
        graph.add_function("test", "test()", 1, "/src/file2.py", file2_id)

        results = graph.find_symbol("test")

        assert len(results) == 2

    def test_get_callers(self, graph):
        """Test finding callers of a function."""
        file_id = graph.add_file("/src/main.py", "python")

        main_id = graph.add_function("main", "main()", 1, "/src/main.py", file_id)
        helper_id = graph.add_function("helper", "helper()", 5, "/src/main.py", file_id)
        util_id = graph.add_function("util", "util()", 10, "/src/main.py", file_id)

        # main calls helper, util calls helper
        graph.add_call_edge(main_id, helper_id)
        graph.add_call_edge(util_id, helper_id)

        callers = graph.get_callers("helper")

        assert len(callers) == 2
        assert "main" in " ".join(callers)
        assert "util" in " ".join(callers)

    def test_get_callers_no_callers(self, populated_graph):
        """Test getting callers when none exist."""
        callers = populated_graph.get_callers("main")

        assert len(callers) == 0


class TestGraphStatistics:
    """Test graph statistics."""

    def test_stats_empty_graph(self, graph):
        """Test statistics on empty graph."""
        stats = graph.stats()

        assert stats["total_nodes"] == 0
        assert stats["total_edges"] == 0

    def test_stats_populated_graph(self, populated_graph):
        """Test statistics on populated graph."""
        stats = populated_graph.stats()

        assert stats["total_nodes"] > 0
        assert stats["total_edges"] > 0
        assert stats["file_nodes"] == 1
        assert stats["function_nodes"] == 2
        assert stats["class_nodes"] == 1
        assert stats["contains_edges"] == 3  # 2 functions + 1 class

    def test_stats_with_multiple_edge_types(self, graph):
        """Test statistics counting different edge types."""
        file_id = graph.add_file("/src/main.py", "python")
        func1_id = graph.add_function("func1", "func1()", 1, "/src/main.py", file_id)
        func2_id = graph.add_function("func2", "func2()", 5, "/src/main.py", file_id)
        import_id = graph.add_import(file_id, "os", [])

        graph.add_call_edge(func1_id, func2_id)

        stats = graph.stats()

        assert stats["contains_edges"] == 2  # file -> func1, file -> func2
        assert stats["calls_edges"] == 1  # func1 -> func2
        assert stats["imports_edges"] == 1  # file -> import


class TestPersistence:
    """Test saving and loading graphs."""

    def test_save_and_load(self, tmp_path, populated_graph):
        """Test saving and loading a graph."""
        save_path = tmp_path / "graph.pkl"

        # Save graph
        populated_graph.save(save_path)
        assert save_path.exists()

        # Create new graph and load
        new_graph = KnowledgeGraph()
        new_graph.load(save_path)

        # Verify data was preserved
        assert new_graph.graph.number_of_nodes() == populated_graph.graph.number_of_nodes()
        assert new_graph.graph.number_of_edges() == populated_graph.graph.number_of_edges()

        # Verify specific data
        contents = new_graph.get_file_contents("/src/main.py")
        assert len(contents["functions"]) == 2
        assert len(contents["classes"]) == 1

    def test_save_creates_directory(self, tmp_path, graph):
        """Test that save creates parent directories."""
        save_path = tmp_path / "nested" / "dir" / "graph.pkl"

        graph.save(save_path)

        assert save_path.exists()
        assert save_path.parent.exists()

    def test_load_preserves_attributes(self, tmp_path, graph):
        """Test that loading preserves all node attributes."""
        file_id = graph.add_file("/src/test.py", "python")
        func_id = graph.add_function(
            name="test",
            signature="test(x, y)",
            line=42,
            file_path="/src/test.py",
            file_node_id=file_id,
        )

        save_path = tmp_path / "graph.pkl"
        graph.save(save_path)

        new_graph = KnowledgeGraph()
        new_graph.load(save_path)

        # Verify all attributes
        node_data = new_graph.graph.nodes[func_id]
        assert node_data["name"] == "test"
        assert node_data["signature"] == "test(x, y)"
        assert node_data["line"] == 42


class TestEdgeCases:
    """Test edge cases and error handling."""

    def test_empty_file_path(self, graph):
        """Test handling empty file path."""
        file_id = graph.add_file("", "python")
        assert file_id == "file:"

    def test_special_characters_in_names(self, graph):
        """Test handling special characters in names."""
        file_id = graph.add_file("/src/test.py", "python")

        # Function with special chars
        func_id = graph.add_function(
            name="__init__",
            signature="__init__(self)",
            line=1,
            file_path="/src/test.py",
            file_node_id=file_id,
        )

        assert graph.graph.has_node(func_id)

    def test_duplicate_additions(self, graph):
        """Test adding the same node multiple times."""
        file_id1 = graph.add_file("/src/test.py", "python")
        file_id2 = graph.add_file("/src/test.py", "python")

        # NetworkX allows duplicate additions (overwrites)
        assert file_id1 == file_id2

    def test_very_long_signature(self, graph):
        """Test handling very long function signatures."""
        file_id = graph.add_file("/src/test.py", "python")

        long_sig = "function(" + ", ".join([f"arg{i}" for i in range(100)]) + ")"

        func_id = graph.add_function(
            name="function",
            signature=long_sig,
            line=1,
            file_path="/src/test.py",
            file_node_id=file_id,
        )

        node_data = graph.graph.nodes[func_id]
        assert node_data["signature"] == long_sig
