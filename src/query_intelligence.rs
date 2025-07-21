use anyhow::Result;
use std::collections::HashMap;
use tracing::{debug, info};

/// Query intelligence for improving search queries
pub struct QueryIntelligence {
    /// Common programming terms and their synonyms
    synonyms: HashMap<String, Vec<String>>,
    /// Common typos and corrections
    typo_corrections: HashMap<String, String>,
    /// Query expansion patterns
    expansion_patterns: Vec<QueryPattern>,
}

#[derive(Debug, Clone)]
struct QueryPattern {
    pattern: String,
    expansions: Vec<String>,
}

impl QueryIntelligence {
    pub fn new() -> Self {
        let mut intelligence = Self {
            synonyms: HashMap::new(),
            typo_corrections: HashMap::new(),
            expansion_patterns: Vec::new(),
        };
        
        intelligence.initialize_synonyms();
        intelligence.initialize_typo_corrections();
        intelligence.initialize_patterns();
        
        intelligence
    }
    
    /// Initialize common programming synonyms
    fn initialize_synonyms(&mut self) {
        // Error handling synonyms
        self.synonyms.insert("error".to_string(), vec![
            "exception".to_string(),
            "failure".to_string(),
            "panic".to_string(),
            "fault".to_string(),
            "bug".to_string(),
        ]);
        
        // Authentication synonyms
        self.synonyms.insert("auth".to_string(), vec![
            "authentication".to_string(),
            "authorization".to_string(),
            "login".to_string(),
            "signin".to_string(),
            "security".to_string(),
        ]);
        
        // Database synonyms
        self.synonyms.insert("database".to_string(), vec![
            "db".to_string(),
            "sql".to_string(),
            "storage".to_string(),
            "persistence".to_string(),
            "repository".to_string(),
        ]);
        
        // Configuration synonyms
        self.synonyms.insert("config".to_string(), vec![
            "configuration".to_string(),
            "settings".to_string(),
            "options".to_string(),
            "preferences".to_string(),
            "setup".to_string(),
        ]);
        
        // Function synonyms
        self.synonyms.insert("function".to_string(), vec![
            "method".to_string(),
            "fn".to_string(),
            "procedure".to_string(),
            "routine".to_string(),
            "callback".to_string(),
        ]);
    }
    
    /// Initialize common typo corrections
    fn initialize_typo_corrections(&mut self) {
        self.typo_corrections.insert("fucntion".to_string(), "function".to_string());
        self.typo_corrections.insert("funciton".to_string(), "function".to_string());
        self.typo_corrections.insert("authentification".to_string(), "authentication".to_string());
        self.typo_corrections.insert("databse".to_string(), "database".to_string());
        self.typo_corrections.insert("conifg".to_string(), "config".to_string());
        self.typo_corrections.insert("hanlder".to_string(), "handler".to_string());
        self.typo_corrections.insert("errer".to_string(), "error".to_string());
        self.typo_corrections.insert("responce".to_string(), "response".to_string());
        self.typo_corrections.insert("reqeust".to_string(), "request".to_string());
    }
    
    /// Initialize query expansion patterns
    fn initialize_patterns(&mut self) {
        // Pattern: "X handling" -> expand to include error handling, exception handling
        self.expansion_patterns.push(QueryPattern {
            pattern: "handling".to_string(),
            expansions: vec![
                "error handling".to_string(),
                "exception handling".to_string(),
                "handle".to_string(),
                "handler".to_string(),
            ],
        });
        
        // Pattern: "X logic" -> expand to include implementation, algorithm
        self.expansion_patterns.push(QueryPattern {
            pattern: "logic".to_string(),
            expansions: vec![
                "implementation".to_string(),
                "algorithm".to_string(),
                "process".to_string(),
                "flow".to_string(),
            ],
        });
    }
    
    /// Suggest query improvements
    pub fn suggest_improvements(&self, query: &str) -> QuerySuggestions {
        let mut suggestions = QuerySuggestions {
            corrected_query: None,
            expanded_queries: Vec::new(),
            related_terms: Vec::new(),
            did_you_mean: None,
        };
        
        // Check for typos
        let corrected = self.correct_typos(query);
        if corrected != query {
            suggestions.corrected_query = Some(corrected.clone());
            suggestions.did_you_mean = Some(format!("Did you mean: {}", corrected));
        }
        
        // Expand query with synonyms
        let expanded = self.expand_with_synonyms(&corrected);
        if !expanded.is_empty() {
            suggestions.expanded_queries = expanded;
        }
        
        // Add related terms
        suggestions.related_terms = self.get_related_terms(&corrected);
        
        suggestions
    }
    
    /// Correct common typos in query
    fn correct_typos(&self, query: &str) -> String {
        let words: Vec<&str> = query.split_whitespace().collect();
        let corrected: Vec<String> = words.iter()
            .map(|word| {
                let lower = word.to_lowercase();
                self.typo_corrections.get(&lower)
                    .cloned()
                    .unwrap_or_else(|| word.to_string())
            })
            .collect();
        
        corrected.join(" ")
    }
    
    /// Expand query with synonyms
    fn expand_with_synonyms(&self, query: &str) -> Vec<String> {
        let mut expansions = Vec::new();
        let words: Vec<&str> = query.split_whitespace().collect();
        
        for word in &words {
            let lower = word.to_lowercase();
            if let Some(synonyms) = self.synonyms.get(&lower) {
                for synonym in synonyms {
                    let mut expanded = words.clone();
                    if let Some(pos) = expanded.iter().position(|&w| w.to_lowercase() == lower) {
                        expanded[pos] = synonym;
                        expansions.push(expanded.join(" "));
                    }
                }
            }
        }
        
        expansions
    }
    
    /// Get related terms for query
    fn get_related_terms(&self, query: &str) -> Vec<String> {
        let mut related = Vec::new();
        let lower_query = query.to_lowercase();
        
        // Check each word for synonyms
        for word in lower_query.split_whitespace() {
            if let Some(synonyms) = self.synonyms.get(word) {
                related.extend(synonyms.clone());
            }
        }
        
        // Check for pattern matches
        for pattern in &self.expansion_patterns {
            if lower_query.contains(&pattern.pattern) {
                related.extend(pattern.expansions.clone());
            }
        }
        
        // Deduplicate
        related.sort();
        related.dedup();
        
        related
    }
    
    /// Analyze query complexity and suggest refinements
    pub fn analyze_query(&self, query: &str) -> QueryAnalysis {
        let word_count = query.split_whitespace().count();
        let has_specific_terms = self.has_specific_terms(query);
        let ambiguity_score = self.calculate_ambiguity(query);
        
        QueryAnalysis {
            complexity: if word_count == 1 {
                QueryComplexity::Simple
            } else if word_count <= 3 {
                QueryComplexity::Moderate
            } else {
                QueryComplexity::Complex
            },
            specificity: if has_specific_terms {
                Specificity::High
            } else if ambiguity_score < 0.5 {
                Specificity::Medium
            } else {
                Specificity::Low
            },
            suggestions: self.generate_refinement_suggestions(query, word_count, ambiguity_score),
        }
    }
    
    /// Check if query contains specific programming terms
    fn has_specific_terms(&self, query: &str) -> bool {
        let specific_terms = [
            "function", "class", "method", "variable", "struct", "enum",
            "interface", "trait", "module", "package", "namespace",
            "async", "await", "promise", "future", "thread", "mutex",
            "error", "exception", "panic", "result", "option",
        ];
        
        let lower_query = query.to_lowercase();
        specific_terms.iter().any(|&term| lower_query.contains(term))
    }
    
    /// Calculate query ambiguity score
    fn calculate_ambiguity(&self, query: &str) -> f32 {
        let vague_terms = ["thing", "stuff", "code", "file", "data", "make", "do", "get", "set"];
        let words: Vec<&str> = query.split_whitespace().collect();
        
        if words.is_empty() {
            return 1.0;
        }
        
        let vague_count = words.iter()
            .filter(|&&word| vague_terms.contains(&word.to_lowercase().as_str()))
            .count();
        
        vague_count as f32 / words.len() as f32
    }
    
    /// Generate refinement suggestions based on analysis
    fn generate_refinement_suggestions(&self, query: &str, word_count: usize, ambiguity: f32) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if word_count == 1 {
            suggestions.push("Try adding more context (e.g., 'error handling' instead of just 'error')".to_string());
        }
        
        if ambiguity > 0.5 {
            suggestions.push("Use more specific terms (e.g., 'parse JSON' instead of 'handle data')".to_string());
        }
        
        if !self.has_specific_terms(query) {
            suggestions.push("Include programming concepts (e.g., function, class, async)".to_string());
        }
        
        suggestions
    }
}

/// Query suggestions from intelligence analysis
#[derive(Debug, Clone)]
pub struct QuerySuggestions {
    pub corrected_query: Option<String>,
    pub expanded_queries: Vec<String>,
    pub related_terms: Vec<String>,
    pub did_you_mean: Option<String>,
}

/// Query complexity analysis
#[derive(Debug, Clone)]
pub struct QueryAnalysis {
    pub complexity: QueryComplexity,
    pub specificity: Specificity,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryComplexity {
    Simple,    // 1 word
    Moderate,  // 2-3 words
    Complex,   // 4+ words
}

#[derive(Debug, Clone, PartialEq)]
pub enum Specificity {
    Low,    // Very vague
    Medium, // Somewhat specific
    High,   // Very specific
}

impl Default for QueryIntelligence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_typo_correction() {
        let qi = QueryIntelligence::new();
        assert_eq!(qi.correct_typos("fucntion implementation"), "function implementation");
        assert_eq!(qi.correct_typos("errer handling"), "error handling");
    }
    
    #[test]
    fn test_synonym_expansion() {
        let qi = QueryIntelligence::new();
        let expansions = qi.expand_with_synonyms("error handling");
        assert!(!expansions.is_empty());
        assert!(expansions.contains(&"exception handling".to_string()));
    }
    
    #[test]
    fn test_query_analysis() {
        let qi = QueryIntelligence::new();
        
        let simple = qi.analyze_query("error");
        assert_eq!(simple.complexity, QueryComplexity::Simple);
        
        let complex = qi.analyze_query("async function error handling implementation");
        assert_eq!(complex.complexity, QueryComplexity::Complex);
        assert_eq!(complex.specificity, Specificity::High);
    }
}