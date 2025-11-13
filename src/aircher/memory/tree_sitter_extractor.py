"""Tree-sitter based code extraction for knowledge graph building."""

from pathlib import Path
from typing import Any

import tree_sitter_python as tspython
import tree_sitter_rust as tsrust
from tree_sitter import Language, Parser


class TreeSitterExtractor:
    """Extract code structure using tree-sitter parsers."""

    def __init__(self):
        """Initialize tree-sitter parsers for supported languages."""
        self.parsers = {}
        self.languages = {}

        # Python
        PY_LANGUAGE = Language(tspython.language())
        py_parser = Parser(PY_LANGUAGE)
        self.parsers["python"] = py_parser
        self.languages["python"] = PY_LANGUAGE

        # Rust
        RUST_LANGUAGE = Language(tsrust.language())
        rust_parser = Parser(RUST_LANGUAGE)
        self.parsers["rust"] = rust_parser
        self.languages["rust"] = RUST_LANGUAGE

        # JavaScript/TypeScript would be added similarly when available
        # JS_LANGUAGE = Language(tsjavascript.language())
        # ...

    def extract_functions(self, file_path: Path, language: str) -> list[dict[str, Any]]:
        """Extract all function definitions from a file.

        Args:
            file_path: Path to the source file.
            language: Programming language (python, rust, javascript, etc.).

        Returns:
            List of function dictionaries with name, signature, line number.
        """
        if language not in self.parsers:
            return []

        try:
            code = file_path.read_bytes()
        except Exception:
            return []

        parser = self.parsers[language]
        tree = parser.parse(code)

        functions = []
        if language == "python":
            functions = self._extract_python_functions(tree.root_node, code)
        elif language == "rust":
            functions = self._extract_rust_functions(tree.root_node, code)

        return functions

    def extract_classes(self, file_path: Path, language: str) -> list[dict[str, Any]]:
        """Extract all class definitions from a file.

        Args:
            file_path: Path to the source file.
            language: Programming language.

        Returns:
            List of class dictionaries with name, line number, methods.
        """
        if language not in self.parsers:
            return []

        try:
            code = file_path.read_bytes()
        except Exception:
            return []

        parser = self.parsers[language]
        tree = parser.parse(code)

        classes = []
        if language == "python":
            classes = self._extract_python_classes(tree.root_node, code)
        elif language == "rust":
            classes = self._extract_rust_structs(tree.root_node, code)

        return classes

    def extract_imports(self, file_path: Path, language: str) -> list[dict[str, Any]]:
        """Extract all import statements from a file.

        Args:
            file_path: Path to the source file.
            language: Programming language.

        Returns:
            List of import dictionaries with module and items.
        """
        if language not in self.parsers:
            return []

        try:
            code = file_path.read_bytes()
        except Exception:
            return []

        parser = self.parsers[language]
        tree = parser.parse(code)

        imports = []
        if language == "python":
            imports = self._extract_python_imports(tree.root_node, code)
        elif language == "rust":
            imports = self._extract_rust_uses(tree.root_node, code)

        return imports

    def _extract_python_functions(self, node: Any, code: bytes) -> list[dict[str, Any]]:
        """Extract Python function definitions."""
        functions = []

        if node.type == "function_definition":
            name_node = node.child_by_field_name("name")
            if name_node:
                name = code[name_node.start_byte : name_node.end_byte].decode("utf-8")
                params_node = node.child_by_field_name("parameters")
                signature = (
                    code[params_node.start_byte : params_node.end_byte].decode("utf-8")
                    if params_node
                    else "()"
                )
                functions.append(
                    {
                        "name": name,
                        "signature": f"{name}{signature}",
                        "line": node.start_point[0] + 1,
                        "end_line": node.end_point[0] + 1,
                    }
                )

        # Recursively process children
        for child in node.children:
            functions.extend(self._extract_python_functions(child, code))

        return functions

    def _extract_python_classes(self, node: Any, code: bytes) -> list[dict[str, Any]]:
        """Extract Python class definitions."""
        classes = []

        if node.type == "class_definition":
            name_node = node.child_by_field_name("name")
            if name_node:
                name = code[name_node.start_byte : name_node.end_byte].decode("utf-8")
                # Extract methods
                methods = []
                body_node = node.child_by_field_name("body")
                if body_node:
                    methods = self._extract_python_functions(body_node, code)

                classes.append(
                    {
                        "name": name,
                        "line": node.start_point[0] + 1,
                        "end_line": node.end_point[0] + 1,
                        "methods": [m["name"] for m in methods],
                    }
                )

        # Recursively process children
        for child in node.children:
            classes.extend(self._extract_python_classes(child, code))

        return classes

    def _extract_python_imports(self, node: Any, code: bytes) -> list[dict[str, Any]]:
        """Extract Python import statements."""
        imports = []

        if node.type == "import_statement":
            # import foo, bar
            for child in node.children:
                if child.type == "dotted_name":
                    module = code[child.start_byte : child.end_byte].decode("utf-8")
                    imports.append({"module": module, "items": []})

        elif node.type == "import_from_statement":
            # from foo import bar, baz
            module_node = node.child_by_field_name("module_name")
            if module_node:
                module = code[module_node.start_byte : module_node.end_byte].decode(
                    "utf-8"
                )
                items = []
                for child in node.children:
                    if child.type == "dotted_name" and child != module_node:
                        item = code[child.start_byte : child.end_byte].decode("utf-8")
                        items.append(item)
                imports.append({"module": module, "items": items})

        # Recursively process children
        for child in node.children:
            imports.extend(self._extract_python_imports(child, code))

        return imports

    def _extract_rust_functions(self, node: Any, code: bytes) -> list[dict[str, Any]]:
        """Extract Rust function definitions."""
        functions = []

        if node.type == "function_item":
            name_node = node.child_by_field_name("name")
            if name_node:
                name = code[name_node.start_byte : name_node.end_byte].decode("utf-8")
                params_node = node.child_by_field_name("parameters")
                signature = (
                    code[params_node.start_byte : params_node.end_byte].decode("utf-8")
                    if params_node
                    else "()"
                )
                functions.append(
                    {
                        "name": name,
                        "signature": f"{name}{signature}",
                        "line": node.start_point[0] + 1,
                        "end_line": node.end_point[0] + 1,
                    }
                )

        # Recursively process children
        for child in node.children:
            functions.extend(self._extract_rust_functions(child, code))

        return functions

    def _extract_rust_structs(self, node: Any, code: bytes) -> list[dict[str, Any]]:
        """Extract Rust struct definitions (treated as classes)."""
        structs = []

        if node.type == "struct_item":
            name_node = node.child_by_field_name("name")
            if name_node:
                name = code[name_node.start_byte : name_node.end_byte].decode("utf-8")
                structs.append(
                    {
                        "name": name,
                        "line": node.start_point[0] + 1,
                        "end_line": node.end_point[0] + 1,
                        "methods": [],  # Would need to find impl blocks
                    }
                )

        # Recursively process children
        for child in node.children:
            structs.extend(self._extract_rust_structs(child, code))

        return structs

    def _extract_rust_uses(self, node: Any, code: bytes) -> list[dict[str, Any]]:
        """Extract Rust use statements."""
        uses = []

        if node.type == "use_declaration":
            # Extract the use path
            use_text = code[node.start_byte : node.end_byte].decode("utf-8")
            # Simple parsing: "use foo::bar;" -> module="foo", items=["bar"]
            if "::" in use_text:
                parts = use_text.replace("use ", "").replace(";", "").split("::")
                if len(parts) >= 2:
                    module = "::".join(parts[:-1])
                    items = [parts[-1]]
                    uses.append({"module": module, "items": items})

        # Recursively process children
        for child in node.children:
            uses.extend(self._extract_rust_uses(child, code))

        return uses
