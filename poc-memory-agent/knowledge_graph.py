#!/usr/bin/env python3
"""
Knowledge Graph Builder for Code Repositories

Extracts code structure (files, classes, functions, imports) using tree-sitter
and builds a queryable graph using NetworkX.

Research hypothesis: Graph queries can guide agents to relevant code faster
than semantic search alone.
"""

import tree_sitter_rust as tsrust
from tree_sitter import Language, Parser, Node
import networkx as nx
from pathlib import Path
from typing import Dict, List, Set, Tuple, Optional
import json


class CodeGraphBuilder:
    """
    Builds a knowledge graph from a Rust codebase.

    Nodes represent:
    - Files (modules)
    - Structs/Enums (types)
    - Functions/Methods
    - Traits

    Edges represent:
    - CONTAINS (module contains function)
    - CALLS (function calls function)
    - USES (function uses type)
    - IMPLEMENTS (type implements trait)
    - IMPORTS (module imports module)
    """

    def __init__(self):
        self.graph = nx.DiGraph()

        # Initialize Rust language and parser
        RUST_LANGUAGE = Language(tsrust.language())
        self.parser = Parser(RUST_LANGUAGE)

        # Track seen entities to avoid duplicates
        self.seen_nodes: Set[str] = set()

    def _node_id(self, node_type: str, name: str, file: str = "") -> str:
        """Generate unique node ID"""
        if file:
            return f"{node_type}:{file}:{name}"
        return f"{node_type}:{name}"

    def _add_node(self, node_id: str, node_type: str, name: str,
                  file: str = "", metadata: Optional[Dict] = None):
        """Add node to graph if not already present"""
        if node_id not in self.seen_nodes:
            self.graph.add_node(
                node_id,
                type=node_type,
                name=name,
                file=file,
                **(metadata or {})
            )
            self.seen_nodes.add(node_id)

    def _add_edge(self, source: str, target: str, edge_type: str,
                  metadata: Optional[Dict] = None):
        """Add edge between nodes"""
        self.graph.add_edge(source, target, type=edge_type, **(metadata or {}))

    def _extract_functions(self, tree: Node, file_path: str, file_id: str):
        """Extract function definitions from AST"""
        query = """
        (function_item
          name: (identifier) @func_name
          parameters: (parameters) @params
        ) @function
        """

        # Tree-sitter query for functions
        for node in tree.root_node.children:
            if node.type == 'function_item':
                func_name_node = node.child_by_field_name('name')
                if func_name_node:
                    func_name = func_name_node.text.decode('utf8')
                    func_id = self._node_id('function', func_name, file_path)

                    # Add function node
                    self._add_node(
                        func_id,
                        'function',
                        func_name,
                        file_path,
                        {'line': func_name_node.start_point[0]}
                    )

                    # Add edge: file CONTAINS function
                    self._add_edge(file_id, func_id, 'contains')

                    # Extract function calls within this function
                    self._extract_calls(node, func_id, file_path)

    def _extract_calls(self, func_node: Node, caller_id: str, file_path: str):
        """Extract function calls within a function body"""
        def traverse(node: Node):
            if node.type == 'call_expression':
                func_node = node.child_by_field_name('function')
                if func_node and func_node.type == 'identifier':
                    callee_name = func_node.text.decode('utf8')
                    # Note: We create a reference to the called function
                    # It might be in another file, but we track the call
                    callee_id = self._node_id('function', callee_name, '')
                    self._add_edge(caller_id, callee_id, 'calls')

            for child in node.children:
                traverse(child)

        traverse(func_node)

    def _extract_structs(self, tree: Node, file_path: str, file_id: str):
        """Extract struct/enum definitions"""
        for node in tree.root_node.children:
            if node.type in ('struct_item', 'enum_item'):
                name_node = node.child_by_field_name('name')
                if name_node:
                    type_name = name_node.text.decode('utf8')
                    type_id = self._node_id('type', type_name, file_path)

                    self._add_node(
                        type_id,
                        'type',
                        type_name,
                        file_path,
                        {'kind': node.type, 'line': name_node.start_point[0]}
                    )

                    # Add edge: file CONTAINS type
                    self._add_edge(file_id, type_id, 'contains')

    def _extract_impl_blocks(self, tree: Node, file_path: str):
        """Extract impl blocks (methods)"""
        for node in tree.root_node.children:
            if node.type == 'impl_item':
                type_node = node.child_by_field_name('type')
                if type_node:
                    # Get the type this impl is for
                    type_name = type_node.text.decode('utf8')
                    type_id = self._node_id('type', type_name, file_path)

                    # Find all functions in this impl
                    body = node.child_by_field_name('body')
                    if body:
                        for child in body.children:
                            if child.type == 'function_item':
                                method_name_node = child.child_by_field_name('name')
                                if method_name_node:
                                    method_name = method_name_node.text.decode('utf8')
                                    method_id = self._node_id(
                                        'method',
                                        f"{type_name}::{method_name}",
                                        file_path
                                    )

                                    self._add_node(
                                        method_id,
                                        'method',
                                        method_name,
                                        file_path,
                                        {'parent_type': type_name, 'line': method_name_node.start_point[0]}
                                    )

                                    # Add edge: type CONTAINS method
                                    self._add_edge(type_id, method_id, 'contains')

                                    # Extract calls within method
                                    self._extract_calls(child, method_id, file_path)

    def _extract_use_statements(self, tree: Node, file_path: str, file_id: str):
        """Extract use/import statements"""
        for node in tree.root_node.children:
            if node.type == 'use_declaration':
                # Extract what's being imported
                # This is simplified - real implementation would parse full paths
                text = node.text.decode('utf8')
                # Example: "use std::collections::HashMap;"
                # We'd create an IMPORTS edge to the imported module
                pass  # Simplified for POC

    def scan_file(self, file_path: Path) -> int:
        """
        Scan a single Rust file and extract code entities.

        Returns: Number of entities extracted
        """
        try:
            code = file_path.read_bytes()
            tree = self.parser.parse(code)

            # Create file node - use absolute path
            file_path_abs = file_path.absolute()
            file_id = self._node_id('file', str(file_path_abs))
            self._add_node(file_id, 'file', file_path.name, str(file_path_abs))

            # Extract entities
            initial_count = len(self.seen_nodes)
            self._extract_functions(tree, str(file_path_abs), file_id)
            self._extract_structs(tree, str(file_path_abs), file_id)
            self._extract_impl_blocks(tree, str(file_path_abs))
            self._extract_use_statements(tree, str(file_path_abs), file_id)

            return len(self.seen_nodes) - initial_count

        except Exception as e:
            print(f"Error scanning {file_path}: {e}")
            return 0

    def scan_repository(self, root: Path, pattern: str = "**/*.rs") -> Dict[str, int]:
        """
        Scan entire repository.

        Args:
            root: Repository root directory
            pattern: Glob pattern for files to scan

        Returns:
            Statistics about scan
        """
        files_scanned = 0
        entities_extracted = 0

        for file_path in root.glob(pattern):
            if file_path.is_file():
                count = self.scan_file(file_path)
                files_scanned += 1
                entities_extracted += count

                if files_scanned % 10 == 0:
                    print(f"Scanned {files_scanned} files, extracted {entities_extracted} entities")

        return {
            'files_scanned': files_scanned,
            'total_nodes': len(self.graph.nodes),
            'total_edges': len(self.graph.edges),
            'node_types': self._count_node_types(),
        }

    def _count_node_types(self) -> Dict[str, int]:
        """Count nodes by type"""
        counts = {}
        for node_id in self.graph.nodes:
            node_data = self.graph.nodes[node_id]
            node_type = node_data.get('type', 'unknown')
            counts[node_type] = counts.get(node_type, 0) + 1
        return counts

    # === Query Interface ===

    def get_file_contents(self, file_path: str) -> Dict[str, List[str]]:
        """
        Get all entities defined in a file.

        Returns: {'functions': [...], 'types': [...], 'methods': [...]}
        """
        file_id = self._node_id('file', file_path)
        result = {'functions': [], 'types': [], 'methods': []}

        if file_id not in self.graph:
            return result

        # Get all nodes contained by this file
        for successor in self.graph.successors(file_id):
            node_data = self.graph.nodes[successor]
            node_type = node_data['type']

            if node_type == 'function':
                result['functions'].append(node_data['name'])
            elif node_type == 'type':
                result['types'].append(node_data['name'])
            elif node_type == 'method':
                result['methods'].append(node_data['name'])

        return result

    def find_callers(self, function_name: str) -> List[str]:
        """Find all functions that call a given function"""
        callers = []

        # Find all nodes with this function name
        for node_id in self.graph.nodes:
            node_data = self.graph.nodes[node_id]
            if node_data['name'] == function_name:
                # Find predecessors (callers)
                for caller_id in self.graph.predecessors(node_id):
                    caller_data = self.graph.nodes[caller_id]
                    if caller_data['type'] in ('function', 'method'):
                        callers.append({
                            'name': caller_data['name'],
                            'file': caller_data.get('file', 'unknown')
                        })

        return callers

    def find_dependencies(self, file_path: str) -> Dict[str, List[str]]:
        """
        Find what a file depends on (calls, uses).

        Returns: {'calls': [...], 'uses': [...]}
        """
        file_id = self._node_id('file', file_path)
        dependencies = {'calls': set(), 'uses': set()}

        if file_id not in self.graph:
            return {'calls': [], 'uses': []}

        # Get all entities in this file
        for entity_id in self.graph.successors(file_id):
            # Find what this entity depends on
            for dep_id in self.graph.successors(entity_id):
                edge_type = self.graph.edges[entity_id, dep_id]['type']
                dep_data = self.graph.nodes[dep_id]

                if edge_type == 'calls':
                    dependencies['calls'].add(dep_data['name'])
                elif edge_type == 'uses':
                    dependencies['uses'].add(dep_data['name'])

        return {
            'calls': list(dependencies['calls']),
            'uses': list(dependencies['uses'])
        }

    def find_related_files(self, file_path: str, max_distance: int = 2) -> List[Tuple[str, int]]:
        """
        Find files related to this one (via calls, imports).

        Returns: List of (file_path, distance) tuples
        """
        file_id = self._node_id('file', file_path)
        if file_id not in self.graph:
            return []

        # BFS to find related files
        related = {}
        visited = {file_id}
        queue = [(file_id, 0)]

        while queue:
            current_id, distance = queue.pop(0)

            if distance >= max_distance:
                continue

            # Explore neighbors
            for neighbor_id in self.graph.successors(current_id):
                neighbor_data = self.graph.nodes[neighbor_id]

                # If neighbor is a file, record it
                if neighbor_data['type'] == 'file' and neighbor_id not in visited:
                    related[neighbor_id] = distance + 1
                    visited.add(neighbor_id)
                    queue.append((neighbor_id, distance + 1))

                # Otherwise, explore its neighbors
                elif neighbor_id not in visited:
                    visited.add(neighbor_id)
                    queue.append((neighbor_id, distance + 1))

        # Convert to list of (file_path, distance)
        return [(self.graph.nodes[fid]['file'], dist)
                for fid, dist in related.items()]

    def export_stats(self) -> Dict:
        """Export graph statistics"""
        return {
            'total_nodes': len(self.graph.nodes),
            'total_edges': len(self.graph.edges),
            'node_types': self._count_node_types(),
            'avg_degree': sum(dict(self.graph.degree()).values()) / len(self.graph.nodes) if self.graph.nodes else 0,
        }

    def save_graph(self, output_path: Path):
        """Save graph to JSON for later analysis"""
        data = {
            'nodes': [
                {
                    'id': node_id,
                    **self.graph.nodes[node_id]
                }
                for node_id in self.graph.nodes
            ],
            'edges': [
                {
                    'source': u,
                    'target': v,
                    **self.graph.edges[u, v]
                }
                for u, v in self.graph.edges
            ]
        }

        output_path.write_text(json.dumps(data, indent=2))


if __name__ == "__main__":
    # Test on Aircher codebase
    builder = CodeGraphBuilder()

    print("Building knowledge graph from Aircher codebase...")
    repo_root = Path("../src")

    stats = builder.scan_repository(repo_root)

    print("\n=== Knowledge Graph Statistics ===")
    for key, value in stats.items():
        print(f"{key}: {value}")

    print("\n=== Export Stats ===")
    for key, value in builder.export_stats().items():
        print(f"{key}: {value}")

    # Example queries
    print("\n=== Example Queries ===")

    # What's in main.rs?
    main_contents = builder.get_file_contents("../src/main.rs")
    print(f"\nmain.rs contains:")
    print(f"  Functions: {main_contents['functions'][:5]}")  # First 5

    # Save graph for later
    builder.save_graph(Path("code_graph.json"))
    print("\nGraph saved to code_graph.json")
