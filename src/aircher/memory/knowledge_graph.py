"""NetworkX-based knowledge graph for code structure and relationships."""

import pickle
from pathlib import Path
from typing import Any

import networkx as nx

from .tree_sitter_extractor import TreeSitterExtractor


class KnowledgeGraph:
    """Knowledge graph of codebase structure using NetworkX."""

    def __init__(self):
        """Initialize an empty directed graph."""
        self.graph = nx.DiGraph()
        self.extractor = TreeSitterExtractor()
        self._node_counter = 0

    def add_file(self, path: str, language: str) -> str:
        """Add a file node to the graph.

        Args:
            path: File path.
            language: Programming language.

        Returns:
            Node ID of the added file.
        """
        node_id = f"file:{path}"
        self.graph.add_node(
            node_id,
            type="file",
            path=path,
            language=language,
        )
        return node_id

    def add_function(
        self,
        name: str,
        signature: str,
        line: int,
        file_path: str,
        file_node_id: str,
    ) -> str:
        """Add a function node to the graph.

        Args:
            name: Function name.
            signature: Full function signature.
            line: Line number in file.
            file_path: Path to the file containing the function.
            file_node_id: Node ID of the containing file.

        Returns:
            Node ID of the added function.
        """
        node_id = f"function:{file_path}:{name}:{line}"
        self.graph.add_node(
            node_id,
            type="function",
            name=name,
            signature=signature,
            line=line,
            file_path=file_path,
        )
        # Add contains edge from file to function
        self.graph.add_edge(file_node_id, node_id, type="contains")
        return node_id

    def add_class(
        self,
        name: str,
        line: int,
        file_path: str,
        file_node_id: str,
        methods: list[str] | None = None,
    ) -> str:
        """Add a class node to the graph.

        Args:
            name: Class name.
            line: Line number in file.
            file_path: Path to the file containing the class.
            file_node_id: Node ID of the containing file.
            methods: List of method names in the class.

        Returns:
            Node ID of the added class.
        """
        node_id = f"class:{file_path}:{name}:{line}"
        self.graph.add_node(
            node_id,
            type="class",
            name=name,
            line=line,
            file_path=file_path,
            methods=methods or [],
        )
        # Add contains edge from file to class
        self.graph.add_edge(file_node_id, node_id, type="contains")
        return node_id

    def add_import(self, file_node_id: str, module: str, items: list[str]) -> str:
        """Add an import node to the graph.

        Args:
            file_node_id: Node ID of the file doing the import.
            module: Module being imported.
            items: Specific items imported (empty for full module import).

        Returns:
            Node ID of the import node.
        """
        node_id = f"import:{file_node_id}:{module}"
        self.graph.add_node(
            node_id,
            type="import",
            module=module,
            items=items,
        )
        # Add imports edge from file to import
        self.graph.add_edge(file_node_id, node_id, type="imports")
        return node_id

    def add_call_edge(self, caller_id: str, callee_id: str) -> None:
        """Add a function call edge.

        Args:
            caller_id: Node ID of the calling function.
            callee_id: Node ID of the called function.
        """
        self.graph.add_edge(caller_id, callee_id, type="calls")

    def build_from_file(self, file_path: Path, language: str) -> str:
        """Build graph nodes from a single file using tree-sitter.

        Args:
            file_path: Path to the source file.
            language: Programming language.

        Returns:
            Node ID of the file.
        """
        # Add file node
        file_node_id = self.add_file(str(file_path), language)

        # Extract and add functions
        functions = self.extractor.extract_functions(file_path, language)
        for func in functions:
            self.add_function(
                name=func["name"],
                signature=func["signature"],
                line=func["line"],
                file_path=str(file_path),
                file_node_id=file_node_id,
            )

        # Extract and add classes
        classes = self.extractor.extract_classes(file_path, language)
        for cls in classes:
            self.add_class(
                name=cls["name"],
                line=cls["line"],
                file_path=str(file_path),
                file_node_id=file_node_id,
                methods=cls.get("methods", []),
            )

        # Extract and add imports
        imports = self.extractor.extract_imports(file_path, language)
        for imp in imports:
            self.add_import(
                file_node_id=file_node_id,
                module=imp["module"],
                items=imp["items"],
            )

        return file_node_id

    def get_file_contents(self, file_path: str) -> dict[str, Any]:
        """Get all functions, classes, and imports in a file.

        Args:
            file_path: Path to the file.

        Returns:
            Dictionary with functions, classes, and imports lists.
        """
        file_node_id = f"file:{file_path}"
        if file_node_id not in self.graph:
            return {"functions": [], "classes": [], "imports": []}

        functions = []
        classes = []
        imports = []

        # Get all nodes connected to this file
        for neighbor in self.graph.neighbors(file_node_id):
            node_data = self.graph.nodes[neighbor]
            edge_type = self.graph.edges[file_node_id, neighbor]["type"]

            if edge_type == "contains":
                if node_data["type"] == "function":
                    functions.append(
                        {
                            "name": node_data["name"],
                            "signature": node_data["signature"],
                            "line": node_data["line"],
                        }
                    )
                elif node_data["type"] == "class":
                    classes.append(
                        {
                            "name": node_data["name"],
                            "line": node_data["line"],
                            "methods": node_data.get("methods", []),
                        }
                    )
            elif edge_type == "imports":
                imports.append(
                    {
                        "module": node_data["module"],
                        "items": node_data["items"],
                    }
                )

        return {
            "functions": functions,
            "classes": classes,
            "imports": imports,
        }

    def get_callers(self, function_name: str) -> list[str]:
        """Find all functions that call a given function.

        Args:
            function_name: Name of the function to search for.

        Returns:
            List of caller function signatures.
        """
        callers = []

        # Find all function nodes with this name
        target_nodes = [
            node
            for node, data in self.graph.nodes(data=True)
            if data.get("type") == "function" and data.get("name") == function_name
        ]

        # Find all predecessors (callers)
        for target in target_nodes:
            for predecessor in self.graph.predecessors(target):
                edge_type = self.graph.edges[predecessor, target].get("type")
                if edge_type == "calls":
                    caller_data = self.graph.nodes[predecessor]
                    if caller_data.get("type") == "function":
                        callers.append(
                            caller_data.get("signature", caller_data["name"])
                        )

        return callers

    def find_symbol(self, name: str) -> list[dict[str, Any]]:
        """Find all occurrences of a symbol (function, class, etc.).

        Args:
            name: Symbol name to search for.

        Returns:
            List of nodes matching the symbol name.
        """
        matches = []
        for node, data in self.graph.nodes(data=True):
            if data.get("name") == name:
                matches.append(
                    {
                        "node_id": node,
                        "type": data["type"],
                        "name": data["name"],
                        "file_path": data.get("file_path"),
                        "line": data.get("line"),
                    }
                )
        return matches

    def save(self, path: Path) -> None:
        """Save the graph to disk using pickle.

        Args:
            path: Path to save the graph.
        """
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("wb") as f:
            pickle.dump(self.graph, f)

    def load(self, path: Path) -> None:
        """Load the graph from disk.

        Args:
            path: Path to load the graph from.
        """
        with path.open("rb") as f:
            self.graph = pickle.load(f)

    def stats(self) -> dict[str, int]:
        """Get statistics about the graph.

        Returns:
            Dictionary with node and edge counts by type.
        """
        stats = {
            "total_nodes": self.graph.number_of_nodes(),
            "total_edges": self.graph.number_of_edges(),
        }

        # Count nodes by type
        for node, data in self.graph.nodes(data=True):
            node_type = data.get("type", "unknown")
            key = f"{node_type}_nodes"
            stats[key] = stats.get(key, 0) + 1

        # Count edges by type
        for _u, _v, data in self.graph.edges(data=True):
            edge_type = data.get("type", "unknown")
            key = f"{edge_type}_edges"
            stats[key] = stats.get(key, 0) + 1

        return stats
