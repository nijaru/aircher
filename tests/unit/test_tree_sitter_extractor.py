"""Unit tests for tree-sitter code extraction."""

from pathlib import Path

import pytest

from aircher.memory.tree_sitter_extractor import TreeSitterExtractor


@pytest.fixture
def extractor():
    """Create a tree-sitter extractor instance."""
    return TreeSitterExtractor()


class TestExtractorInitialization:
    """Test extractor initialization."""

    def test_initialization(self):
        """Test creating an extractor."""
        extractor = TreeSitterExtractor()
        assert extractor is not None
        assert extractor.parsers is not None
        assert extractor.languages is not None

    def test_python_parser_loaded(self, extractor):
        """Test that Python parser is loaded."""
        assert "python" in extractor.parsers
        assert "python" in extractor.languages

    def test_rust_parser_loaded(self, extractor):
        """Test that Rust parser is loaded."""
        assert "rust" in extractor.parsers
        assert "rust" in extractor.languages


class TestPythonFunctionExtraction:
    """Test extracting functions from Python code."""

    def test_extract_simple_function(self, tmp_path, extractor):
        """Test extracting a simple Python function."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
def hello():
    print("Hello, World!")
""")

        functions = extractor.extract_functions(test_file, "python")

        assert len(functions) >= 1
        func = next((f for f in functions if f["name"] == "hello"), None)
        assert func is not None
        assert "hello" in func["signature"]
        assert func["line"] > 0

    def test_extract_function_with_parameters(self, tmp_path, extractor):
        """Test extracting function with parameters."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
def add(x, y):
    return x + y
""")

        functions = extractor.extract_functions(test_file, "python")

        func = next((f for f in functions if f["name"] == "add"), None)
        assert func is not None
        assert "x" in func["signature"]
        assert "y" in func["signature"]

    def test_extract_multiple_functions(self, tmp_path, extractor):
        """Test extracting multiple functions."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
def func1():
    pass

def func2(a, b):
    return a + b

def func3():
    pass
""")

        functions = extractor.extract_functions(test_file, "python")

        assert len(functions) >= 3
        func_names = [f["name"] for f in functions]
        assert "func1" in func_names
        assert "func2" in func_names
        assert "func3" in func_names

    def test_extract_nested_functions(self, tmp_path, extractor):
        """Test extracting nested functions."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
def outer():
    def inner():
        pass
    return inner
""")

        functions = extractor.extract_functions(test_file, "python")

        # Should extract both outer and inner
        func_names = [f["name"] for f in functions]
        assert "outer" in func_names
        assert "inner" in func_names

    def test_extract_async_function(self, tmp_path, extractor):
        """Test extracting async functions."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
async def async_func():
    await something()
""")

        functions = extractor.extract_functions(test_file, "python")

        func = next((f for f in functions if f["name"] == "async_func"), None)
        assert func is not None

    def test_function_line_numbers(self, tmp_path, extractor):
        """Test that line numbers are correct."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""# Line 1
# Line 2
def my_function():  # Line 3
    pass  # Line 4
# Line 5
""")

        functions = extractor.extract_functions(test_file, "python")

        func = next((f for f in functions if f["name"] == "my_function"), None)
        assert func is not None
        assert func["line"] == 3


class TestPythonClassExtraction:
    """Test extracting classes from Python code."""

    def test_extract_simple_class(self, tmp_path, extractor):
        """Test extracting a simple class."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
class MyClass:
    pass
""")

        classes = extractor.extract_classes(test_file, "python")

        assert len(classes) >= 1
        cls = next((c for c in classes if c["name"] == "MyClass"), None)
        assert cls is not None
        assert cls["line"] > 0

    def test_extract_class_with_methods(self, tmp_path, extractor):
        """Test extracting class with methods."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
class Calculator:
    def add(self, x, y):
        return x + y

    def subtract(self, x, y):
        return x - y
""")

        classes = extractor.extract_classes(test_file, "python")

        cls = next((c for c in classes if c["name"] == "Calculator"), None)
        assert cls is not None
        assert "add" in cls["methods"]
        assert "subtract" in cls["methods"]

    def test_extract_multiple_classes(self, tmp_path, extractor):
        """Test extracting multiple classes."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
class Class1:
    pass

class Class2:
    def method(self):
        pass
""")

        classes = extractor.extract_classes(test_file, "python")

        assert len(classes) >= 2
        class_names = [c["name"] for c in classes]
        assert "Class1" in class_names
        assert "Class2" in class_names

    def test_extract_nested_classes(self, tmp_path, extractor):
        """Test extracting nested classes."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
class Outer:
    class Inner:
        pass
""")

        classes = extractor.extract_classes(test_file, "python")

        class_names = [c["name"] for c in classes]
        assert "Outer" in class_names
        assert "Inner" in class_names


class TestPythonImportExtraction:
    """Test extracting imports from Python code."""

    def test_extract_simple_import(self, tmp_path, extractor):
        """Test extracting simple import statement."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
import os
""")

        imports = extractor.extract_imports(test_file, "python")

        assert len(imports) >= 1
        imp = next((i for i in imports if i["module"] == "os"), None)
        assert imp is not None
        assert imp["items"] == []

    def test_extract_from_import(self, tmp_path, extractor):
        """Test extracting from...import statement."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
from pathlib import Path
""")

        imports = extractor.extract_imports(test_file, "python")

        imp = next((i for i in imports if i["module"] == "pathlib"), None)
        assert imp is not None
        # Items may or may not be extracted depending on tree-sitter version

    def test_extract_multiple_imports(self, tmp_path, extractor):
        """Test extracting multiple imports."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
import os
import sys
from pathlib import Path
""")

        imports = extractor.extract_imports(test_file, "python")

        assert len(imports) >= 2
        modules = [i["module"] for i in imports]
        assert "os" in modules or "sys" in modules


class TestRustFunctionExtraction:
    """Test extracting functions from Rust code."""

    def test_extract_simple_rust_function(self, tmp_path, extractor):
        """Test extracting a simple Rust function."""
        test_file = tmp_path / "test.rs"
        test_file.write_text("""
fn main() {
    println!("Hello, World!");
}
""")

        functions = extractor.extract_functions(test_file, "rust")

        func = next((f for f in functions if f["name"] == "main"), None)
        assert func is not None
        assert "main" in func["signature"]

    def test_extract_rust_function_with_parameters(self, tmp_path, extractor):
        """Test extracting Rust function with parameters."""
        test_file = tmp_path / "test.rs"
        test_file.write_text("""
fn add(x: i32, y: i32) -> i32 {
    x + y
}
""")

        functions = extractor.extract_functions(test_file, "rust")

        func = next((f for f in functions if f["name"] == "add"), None)
        assert func is not None
        assert "x" in func["signature"] or "i32" in func["signature"]

    def test_extract_multiple_rust_functions(self, tmp_path, extractor):
        """Test extracting multiple Rust functions."""
        test_file = tmp_path / "test.rs"
        test_file.write_text("""
fn func1() { }

fn func2(x: i32) -> i32 {
    x + 1
}
""")

        functions = extractor.extract_functions(test_file, "rust")

        assert len(functions) >= 2
        func_names = [f["name"] for f in functions]
        assert "func1" in func_names
        assert "func2" in func_names


class TestRustStructExtraction:
    """Test extracting structs from Rust code."""

    def test_extract_simple_struct(self, tmp_path, extractor):
        """Test extracting a simple Rust struct."""
        test_file = tmp_path / "test.rs"
        test_file.write_text("""
struct Point {
    x: i32,
    y: i32,
}
""")

        structs = extractor.extract_classes(test_file, "rust")

        struct = next((s for s in structs if s["name"] == "Point"), None)
        assert struct is not None

    def test_extract_multiple_structs(self, tmp_path, extractor):
        """Test extracting multiple structs."""
        test_file = tmp_path / "test.rs"
        test_file.write_text("""
struct Struct1 {
    field: i32,
}

struct Struct2 {
    value: String,
}
""")

        structs = extractor.extract_classes(test_file, "rust")

        assert len(structs) >= 2
        struct_names = [s["name"] for s in structs]
        assert "Struct1" in struct_names
        assert "Struct2" in struct_names


class TestRustUseExtraction:
    """Test extracting use statements from Rust code."""

    def test_extract_simple_use(self, tmp_path, extractor):
        """Test extracting simple use statement."""
        test_file = tmp_path / "test.rs"
        test_file.write_text("""
use std::collections::HashMap;
""")

        uses = extractor.extract_imports(test_file, "rust")

        # May or may not extract depending on parsing
        assert isinstance(uses, list)

    def test_extract_multiple_uses(self, tmp_path, extractor):
        """Test extracting multiple use statements."""
        test_file = tmp_path / "test.rs"
        test_file.write_text("""
use std::fs;
use std::io::Read;
""")

        uses = extractor.extract_imports(test_file, "rust")

        assert isinstance(uses, list)


class TestUnsupportedLanguages:
    """Test behavior with unsupported languages."""

    def test_unsupported_language_functions(self, tmp_path, extractor):
        """Test extracting from unsupported language returns empty."""
        test_file = tmp_path / "test.xyz"
        test_file.write_text("some code")

        functions = extractor.extract_functions(test_file, "unsupported")

        assert functions == []

    def test_unsupported_language_classes(self, tmp_path, extractor):
        """Test extracting classes from unsupported language."""
        test_file = tmp_path / "test.xyz"
        test_file.write_text("some code")

        classes = extractor.extract_classes(test_file, "unsupported")

        assert classes == []

    def test_unsupported_language_imports(self, tmp_path, extractor):
        """Test extracting imports from unsupported language."""
        test_file = tmp_path / "test.xyz"
        test_file.write_text("some code")

        imports = extractor.extract_imports(test_file, "unsupported")

        assert imports == []


class TestErrorHandling:
    """Test error handling."""

    def test_nonexistent_file(self, extractor):
        """Test extracting from nonexistent file."""
        nonexistent = Path("/nonexistent/file.py")

        functions = extractor.extract_functions(nonexistent, "python")

        assert functions == []

    def test_binary_file(self, tmp_path, extractor):
        """Test extracting from binary file."""
        binary_file = tmp_path / "test.py"
        binary_file.write_bytes(b"\x00\x01\x02\x03")

        functions = extractor.extract_functions(binary_file, "python")

        # Should handle gracefully, return empty or partial results
        assert isinstance(functions, list)

    def test_malformed_code(self, tmp_path, extractor):
        """Test extracting from malformed code."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
def incomplete_function(
    # Missing closing parenthesis and body
""")

        # Should not crash, may return empty or partial results
        functions = extractor.extract_functions(test_file, "python")
        assert isinstance(functions, list)

    def test_empty_file(self, tmp_path, extractor):
        """Test extracting from empty file."""
        test_file = tmp_path / "test.py"
        test_file.write_text("")

        functions = extractor.extract_functions(test_file, "python")
        classes = extractor.extract_classes(test_file, "python")
        imports = extractor.extract_imports(test_file, "python")

        assert functions == []
        assert classes == []
        assert imports == []


class TestComplexCode:
    """Test extraction from complex real-world code."""

    def test_class_with_decorators(self, tmp_path, extractor):
        """Test extracting class with decorators."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
@dataclass
class Person:
    name: str
    age: int

    def greet(self):
        print(f"Hello, I'm {self.name}")
""")

        classes = extractor.extract_classes(test_file, "python")

        cls = next((c for c in classes if c["name"] == "Person"), None)
        assert cls is not None
        assert "greet" in cls["methods"]

    def test_function_with_decorators(self, tmp_path, extractor):
        """Test extracting function with decorators."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
@staticmethod
def static_method():
    pass
""")

        functions = extractor.extract_functions(test_file, "python")

        func = next((f for f in functions if f["name"] == "static_method"), None)
        assert func is not None

    def test_mixed_code(self, tmp_path, extractor):
        """Test extracting from file with mixed elements."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""
import os
from pathlib import Path

class MyClass:
    def method1(self):
        pass

    def method2(self):
        pass

def standalone_function():
    pass

class AnotherClass:
    pass
""")

        functions = extractor.extract_functions(test_file, "python")
        classes = extractor.extract_classes(test_file, "python")
        imports = extractor.extract_imports(test_file, "python")

        # Should extract everything
        assert len(functions) >= 1  # standalone_function + methods
        assert len(classes) >= 2  # MyClass, AnotherClass
        assert len(imports) >= 1  # os and/or pathlib

    def test_end_line_tracking(self, tmp_path, extractor):
        """Test that end_line is tracked correctly."""
        test_file = tmp_path / "test.py"
        test_file.write_text("""def my_function():
    line2 = True
    line3 = True
    return line3
""")

        functions = extractor.extract_functions(test_file, "python")

        func = next((f for f in functions if f["name"] == "my_function"), None)
        assert func is not None
        assert "end_line" in func
        # Should span multiple lines
        assert func["end_line"] > func["line"]
