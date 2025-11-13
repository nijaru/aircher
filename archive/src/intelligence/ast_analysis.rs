//! AST Analysis Module for Intelligence Engine
//!
//! Provides syntactic and semantic code analysis using tree-sitter
//! to enhance the intelligence system's understanding of code structure,
//! patterns, and relationships.

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Query, QueryCursor, Tree};
use streaming_iterator::StreamingIterator;
use serde::{Serialize, Deserialize};
use tracing::{debug, warn};

/// AST analysis results for code structure understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTAnalysis {
    pub file_path: PathBuf,
    pub language: String,
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub imports: Vec<ImportInfo>,
    pub exports: Vec<ExportInfo>,
    pub complexity_metrics: ComplexityMetrics,
    pub patterns: Vec<CodePattern>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub visibility: Visibility,
    pub is_async: bool,
    pub complexity: u32,
    pub calls: Vec<String>, // Functions this function calls
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub methods: Vec<FunctionInfo>,
    pub fields: Vec<FieldInfo>,
    pub inheritance: Vec<String>,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<String>,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: Option<String>,
    pub visibility: Visibility,
    pub is_static: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    pub module: String,
    pub items: Vec<String>, // Empty for wildcard imports
    pub is_wildcard: bool,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportInfo {
    pub name: String,
    pub export_type: ExportType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportType {
    Function,
    Class,
    Variable,
    Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub nesting_depth: u32,
    pub lines_of_code: u32,
    pub comment_ratio: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePattern {
    pub pattern_type: String,
    pub description: String,
    pub location: (usize, usize), // start_line, end_line
    pub confidence: f32,
}

/// Language-specific AST analyzer
pub struct ASTAnalyzer {
    parsers: HashMap<String, Parser>,
    queries: HashMap<String, Vec<Query>>,
}

impl ASTAnalyzer {
    pub fn new() -> Result<Self> {
        let mut analyzer = Self {
            parsers: HashMap::new(),
            queries: HashMap::new(),
        };

        // Initialize parsers for supported languages
        analyzer.init_language_support()?;

        Ok(analyzer)
    }

    /// Initialize parser and query support for various languages
    fn init_language_support(&mut self) -> Result<()> {
        // Rust support
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
        self.parsers.insert("rust".to_string(), parser);
        self.init_rust_queries()?;

        // Python support
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::LANGUAGE.into())?;
        self.parsers.insert("python".to_string(), parser);
        self.init_python_queries()?;

        // JavaScript/TypeScript support
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_javascript::LANGUAGE.into())?;
        self.parsers.insert("javascript".to_string(), parser);
        self.init_javascript_queries()?;

        // Add more languages as needed
        debug!("Initialized AST analysis for {} languages", self.parsers.len());
        Ok(())
    }

    /// Analyze a code file and extract AST information
    pub async fn analyze_file(&mut self, file_path: &Path) -> Result<Option<ASTAnalysis>> {
        let language = self.detect_language(file_path)?;

        if !self.parsers.contains_key(&language) {
            debug!("Unsupported language: {} for file: {:?}", language, file_path);
            return Ok(None);
        }

        let code = tokio::fs::read_to_string(file_path).await?;
        self.analyze_code(&code, file_path, &language).await
    }

    /// Analyze code string directly
    pub async fn analyze_code(&mut self, code: &str, file_path: &Path, language: &str) -> Result<Option<ASTAnalysis>> {
        let parser = self.parsers.get_mut(language);
        if parser.is_none() {
            return Ok(None);
        }

        let parser = parser.unwrap();
        let tree = parser.parse(code, None);

        if tree.is_none() {
            warn!("Failed to parse code for file: {:?}", file_path);
            return Ok(None);
        }

        let tree = tree.unwrap();
        let analysis = self.extract_analysis(code, &tree, file_path, language).await?;

        Ok(Some(analysis))
    }

    /// Extract comprehensive analysis from parsed tree
    async fn extract_analysis(&self, code: &str, tree: &Tree, file_path: &Path, language: &str) -> Result<ASTAnalysis> {
        let mut analysis = ASTAnalysis {
            file_path: file_path.to_path_buf(),
            language: language.to_string(),
            functions: Vec::new(),
            classes: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            complexity_metrics: ComplexityMetrics {
                cyclomatic_complexity: 0,
                cognitive_complexity: 0,
                nesting_depth: 0,
                lines_of_code: code.lines().count() as u32,
                comment_ratio: 0.0,
            },
            patterns: Vec::new(),
            dependencies: Vec::new(),
        };

        // Extract functions, classes, imports based on language
        match language {
            "rust" => self.analyze_rust(code, tree, &mut analysis).await?,
            "python" => self.analyze_python(code, tree, &mut analysis).await?,
            "javascript" | "typescript" => self.analyze_javascript(code, tree, &mut analysis).await?,
            _ => {
                // Generic analysis for unsupported languages
                self.analyze_generic(code, tree, &mut analysis).await?;
            }
        }

        // Calculate complexity metrics
        self.calculate_complexity_metrics(code, tree, &mut analysis).await?;

        // Detect code patterns
        self.detect_code_patterns(code, tree, &mut analysis).await?;

        Ok(analysis)
    }

    /// Rust-specific AST analysis
    async fn analyze_rust(&self, code: &str, tree: &Tree, analysis: &mut ASTAnalysis) -> Result<()> {
        let queries = self.queries.get("rust");
        if queries.is_none() {
            return Ok(());
        }

        let mut cursor = QueryCursor::new();
        let root_node = tree.root_node();

        // Extract functions
        for query in queries.unwrap() {
            let mut matches = cursor.matches(query, root_node, code.as_bytes());

            while let Some(match_) = matches.next() {
                for capture in match_.captures {
                    let node = capture.node;
                    let _text = node.utf8_text(code.as_bytes())?;

                    // Parse function information
                    if let Ok(function_info) = self.parse_rust_function(&node, code) {
                        analysis.functions.push(function_info);
                    }

                    // Parse struct/impl blocks as classes
                    if let Ok(class_info) = self.parse_rust_struct(&node, code) {
                        analysis.classes.push(class_info);
                    }

                    // Parse use statements as imports
                    if let Ok(import_info) = self.parse_rust_import(&node, code) {
                        analysis.imports.push(import_info);
                    }
                }
            }
        }

        Ok(())
    }

    /// Python-specific AST analysis
    async fn analyze_python(&self, _code: &str, _tree: &Tree, _analysis: &mut ASTAnalysis) -> Result<()> {
        // Similar implementation for Python
        // Extract function definitions, class definitions, import statements
        Ok(())
    }

    /// JavaScript/TypeScript-specific AST analysis
    async fn analyze_javascript(&self, _code: &str, _tree: &Tree, _analysis: &mut ASTAnalysis) -> Result<()> {
        // Similar implementation for JavaScript/TypeScript
        // Extract function declarations, class declarations, import/export statements
        Ok(())
    }

    /// Generic analysis for unsupported languages
    async fn analyze_generic(&self, _code: &str, _tree: &Tree, _analysis: &mut ASTAnalysis) -> Result<()> {
        // Basic pattern matching for common structures
        Ok(())
    }

    /// Calculate complexity metrics
    async fn calculate_complexity_metrics(&self, code: &str, _tree: &Tree, analysis: &mut ASTAnalysis) -> Result<()> {
        let lines = code.lines().collect::<Vec<_>>();
        let comment_lines = lines.iter().filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("//") || trimmed.starts_with("#") ||
            trimmed.starts_with("/*") || trimmed.starts_with("*")
        }).count();

        analysis.complexity_metrics.comment_ratio = if lines.len() > 0 {
            comment_lines as f32 / lines.len() as f32
        } else {
            0.0
        };

        // TODO: Implement cyclomatic and cognitive complexity calculation

        Ok(())
    }

    /// Detect common code patterns
    async fn detect_code_patterns(&self, _code: &str, _tree: &Tree, _analysis: &mut ASTAnalysis) -> Result<()> {
        // Detect common patterns like:
        // - Error handling patterns
        // - Design patterns (Factory, Builder, etc.)
        // - Anti-patterns
        // - Performance patterns
        // - Security patterns

        Ok(())
    }

    /// Detect programming language from file extension
    fn detect_language(&self, file_path: &Path) -> Result<String> {
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let language = match extension {
            "rs" => "rust",
            "py" | "pyi" => "python",
            "js" | "jsx" => "javascript",
            "ts" | "tsx" => "typescript",
            "go" => "go",
            "c" | "h" => "c",
            "cpp" | "cxx" | "cc" | "hpp" | "hxx" => "cpp",
            "java" => "java",
            "cs" => "c_sharp",
            "php" => "php",
            "rb" => "ruby",
            "swift" => "swift",
            "kt" | "kts" => "kotlin",
            "yaml" | "yml" => "yaml",
            "sql" => "sql",
            "json" => "json",
            "sh" | "bash" => "bash",
            "html" | "htm" => "html",
            "css" => "css",
            _ => "unknown",
        };

        Ok(language.to_string())
    }

    /// Initialize Rust-specific queries
    fn init_rust_queries(&mut self) -> Result<()> {
        let queries = vec![
            // Function definitions
            Query::new(&tree_sitter_rust::LANGUAGE.into(), "(function_item) @function")?,
            // Struct definitions
            Query::new(&tree_sitter_rust::LANGUAGE.into(), "(struct_item) @struct")?,
            // Impl blocks
            Query::new(&tree_sitter_rust::LANGUAGE.into(), "(impl_item) @impl")?,
            // Use statements
            Query::new(&tree_sitter_rust::LANGUAGE.into(), "(use_declaration) @use")?,
        ];

        self.queries.insert("rust".to_string(), queries);
        Ok(())
    }

    /// Initialize Python-specific queries
    fn init_python_queries(&mut self) -> Result<()> {
        let queries = vec![
            // Function definitions
            Query::new(&tree_sitter_python::LANGUAGE.into(), "(function_definition) @function")?,
            // Class definitions
            Query::new(&tree_sitter_python::LANGUAGE.into(), "(class_definition) @class")?,
            // Import statements
            Query::new(&tree_sitter_python::LANGUAGE.into(), "(import_statement) @import")?,
            Query::new(&tree_sitter_python::LANGUAGE.into(), "(import_from_statement) @import_from")?,
        ];

        self.queries.insert("python".to_string(), queries);
        Ok(())
    }

    /// Initialize JavaScript-specific queries
    fn init_javascript_queries(&mut self) -> Result<()> {
        let queries = vec![
            // Function declarations
            Query::new(&tree_sitter_javascript::LANGUAGE.into(), "(function_declaration) @function")?,
            // Class declarations
            Query::new(&tree_sitter_javascript::LANGUAGE.into(), "(class_declaration) @class")?,
            // Import statements
            Query::new(&tree_sitter_javascript::LANGUAGE.into(), "(import_statement) @import")?,
        ];

        self.queries.insert("javascript".to_string(), queries);
        Ok(())
    }

    /// Parse Rust function information
    fn parse_rust_function(&self, node: &tree_sitter::Node, _code: &str) -> Result<FunctionInfo> {
        let function_name = "unknown"; // TODO: Extract from AST node

        Ok(FunctionInfo {
            name: function_name.to_string(),
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            parameters: Vec::new(), // TODO: Parse parameters
            return_type: None, // TODO: Parse return type
            visibility: Visibility::Unknown, // TODO: Parse visibility
            is_async: false, // TODO: Detect async
            complexity: 1, // TODO: Calculate complexity
            calls: Vec::new(), // TODO: Extract function calls
        })
    }

    /// Parse Rust struct information
    fn parse_rust_struct(&self, node: &tree_sitter::Node, _code: &str) -> Result<ClassInfo> {
        let struct_name = "unknown"; // TODO: Extract from AST node

        Ok(ClassInfo {
            name: struct_name.to_string(),
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            methods: Vec::new(), // TODO: Parse methods from impl blocks
            fields: Vec::new(), // TODO: Parse fields
            inheritance: Vec::new(), // Rust doesn't have inheritance, but traits
            visibility: Visibility::Unknown,
        })
    }

    /// Parse Rust import information
    fn parse_rust_import(&self, _node: &tree_sitter::Node, _code: &str) -> Result<ImportInfo> {
        let module_name = "unknown"; // TODO: Extract from AST node

        Ok(ImportInfo {
            module: module_name.to_string(),
            items: Vec::new(), // TODO: Parse imported items
            is_wildcard: false, // TODO: Detect wildcard imports
            alias: None, // TODO: Parse aliases
        })
    }
}

impl Default for ASTAnalyzer {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            parsers: HashMap::new(),
            queries: HashMap::new(),
        })
    }
}

/// Utility functions for AST analysis
pub mod utils {
    use super::*;

    /// Get function signatures from AST analysis
    pub fn extract_function_signatures(analysis: &ASTAnalysis) -> Vec<String> {
        analysis.functions.iter().map(|func| {
            let params = func.parameters.iter()
                .map(|p| format!("{}: {}", p.name, p.param_type.as_deref().unwrap_or("unknown")))
                .collect::<Vec<_>>()
                .join(", ");

            let return_type = func.return_type.as_deref().unwrap_or("()");
            format!("fn {}({}) -> {}", func.name, params, return_type)
        }).collect()
    }

    /// Calculate overall code quality score
    pub fn calculate_quality_score(analysis: &ASTAnalysis) -> f32 {
        let metrics = &analysis.complexity_metrics;

        // Simple quality score based on various factors
        let complexity_score = 1.0 - (metrics.cyclomatic_complexity as f32 / 20.0).min(1.0);
        let comment_score = metrics.comment_ratio.min(1.0);
        let pattern_score = analysis.patterns.iter()
            .map(|p| p.confidence)
            .sum::<f32>() / analysis.patterns.len().max(1) as f32;

        (complexity_score + comment_score + pattern_score) / 3.0
    }

    /// Extract dependencies from imports
    pub fn extract_dependencies(analysis: &ASTAnalysis) -> Vec<String> {
        let mut deps: Vec<String> = analysis.imports.iter()
            .map(|import| import.module.clone())
            .collect();

        deps.sort();
        deps.dedup();
        deps
    }
}
