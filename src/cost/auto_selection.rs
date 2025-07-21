use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use super::{EmbeddingManager, EmbeddingModel};

/// Advanced embedding model auto-selection engine
pub struct AutoSelectionEngine {
    performance_cache: HashMap<String, ModelPerformance>,
    system_info: SystemCapabilities,
    task_preferences: TaskPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    pub model_name: String,
    pub avg_response_time_ms: f64,
    pub reliability_score: f64,  // 0.0-1.0
    pub last_successful_use: chrono::DateTime<chrono::Utc>,
    pub consecutive_failures: u32,
    pub embedding_quality_score: f64,  // Based on task-specific performance
}

#[derive(Debug, Clone)]
pub struct SystemCapabilities {
    pub available_memory_mb: u64,
    pub cpu_cores: usize,
    pub network_available: bool,
    pub ollama_running: bool,
    pub has_gpu: bool,
}

#[derive(Debug, Clone)]
pub struct TaskPreferences {
    preferences: HashMap<TaskType, Vec<ModelPreference>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskType {
    CodeSearch,
    DocumentationSearch,
    ConceptualSearch,
    CrossLanguageSearch,
    LargeCodebaseSearch,
    QuickSearch,
    HighAccuracySearch,
}

#[derive(Debug, Clone)]
pub struct ModelPreference {
    pub model_name: String,
    pub provider: String,
    pub preference_score: f64,  // 0.0-1.0
    pub min_memory_mb: u64,
    pub reasoning: String,
}

#[derive(Debug, Clone)]
pub struct SelectionCriteria {
    pub task_type: TaskType,
    pub max_response_time_ms: Option<u64>,
    pub min_reliability: Option<f64>,
    pub prefer_local: bool,
    pub prefer_fast: bool,
    pub prefer_accurate: bool,
}

impl Default for SelectionCriteria {
    fn default() -> Self {
        Self {
            task_type: TaskType::CodeSearch,
            max_response_time_ms: Some(5000),
            min_reliability: Some(0.8),
            prefer_local: true,
            prefer_fast: false,
            prefer_accurate: true,
        }
    }
}

impl AutoSelectionEngine {
    pub fn new() -> Self {
        let system_info = Self::detect_system_capabilities();
        let task_preferences = Self::initialize_task_preferences();
        
        Self {
            performance_cache: HashMap::new(),
            system_info,
            task_preferences,
        }
    }
    
    /// Intelligently select the best embedding model based on criteria
    pub async fn select_optimal_model(
        &mut self, 
        manager: &mut EmbeddingManager,
        criteria: &SelectionCriteria
    ) -> Result<EmbeddingModel> {
        info!("Starting intelligent model selection for task: {:?}", criteria.task_type);
        
        // Get available models
        let available_models = self.get_available_models(manager).await?;
        if available_models.is_empty() {
            anyhow::bail!("No embedding models available");
        }
        
        // Score all available models
        let scored_models = self.score_models(&available_models, criteria).await?;
        
        // Select the highest scoring model
        let best_model = scored_models.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(model, score)| {
                info!("Selected model '{}' with score {:.3}", model.name, score);
                model
            })
            .context("No suitable model found")?;
        
        // Test the selected model
        self.verify_model_health(manager, &best_model).await?;
        
        Ok(best_model)
    }
    
    async fn get_available_models(&self, manager: &mut EmbeddingManager) -> Result<Vec<EmbeddingModel>> {
        let mut available = Vec::new();
        
        // Check embedded models
        if manager.is_embedded_model_available().await {
            available.extend(
                EmbeddingManager::get_coding_optimized_models()
                    .into_iter()
                    .filter(|m| m.provider == "embedded")
            );
        }
        
        // Check Ollama models
        if manager.check_ollama_availability().await {
            for model in EmbeddingManager::get_coding_optimized_models() {
                if model.provider == "ollama" {
                    // For now, assume all Ollama models are available if Ollama is running
                    // In a real implementation, we would check individual model availability
                    available.push(model);
                }
            }
        }
        
        // Add API-based models (always "available" if we have network)
        if self.system_info.network_available {
            available.extend(
                EmbeddingManager::get_coding_optimized_models()
                    .into_iter()
                    .filter(|m| m.provider == "huggingface")
            );
        }
        
        debug!("Found {} available embedding models", available.len());
        Ok(available)
    }
    
    async fn score_models(
        &self, 
        models: &[EmbeddingModel], 
        criteria: &SelectionCriteria
    ) -> Result<Vec<(EmbeddingModel, f64)>> {
        let mut scored = Vec::new();
        
        for model in models {
            let score = self.calculate_model_score(model, criteria).await?;
            if score > 0.0 {  // Only include models with positive scores
                scored.push((model.clone(), score));
            }
        }
        
        Ok(scored)
    }
    
    async fn calculate_model_score(
        &self, 
        model: &EmbeddingModel, 
        criteria: &SelectionCriteria
    ) -> Result<f64> {
        let mut score = 0.0;
        
        // Base score from task preferences
        if let Some(task_prefs) = self.task_preferences.preferences.get(&criteria.task_type) {
            if let Some(pref) = task_prefs.iter().find(|p| p.model_name == model.name) {
                score += pref.preference_score * 40.0;  // Up to 40 points for task suitability
            }
        }
        
        // Provider preference scoring
        score += match model.provider.as_str() {
            "embedded" => {
                if criteria.prefer_local { 35.0 } else { 25.0 }  // Embedded gets priority
            },
            "ollama" => {
                if criteria.prefer_local { 30.0 } else { 20.0 }  // Ollama second for local
            },
            "huggingface" => {
                if criteria.prefer_local { 10.0 } else { 15.0 }  // API fallback
            },
            _ => 5.0,
        };
        
        // Performance scoring from cache
        if let Some(perf) = self.performance_cache.get(&model.name) {
            // Reliability score (0-15 points)
            score += perf.reliability_score * 15.0;
            
            // Response time score (0-10 points)
            if let Some(max_time) = criteria.max_response_time_ms {
                let time_score = if perf.avg_response_time_ms <= max_time as f64 {
                    10.0 * (1.0 - (perf.avg_response_time_ms / max_time as f64))
                } else {
                    0.0  // Exceeds max time
                };
                score += time_score;
            }
            
            // Penalize models with recent failures
            score -= perf.consecutive_failures as f64 * 5.0;
        } else {
            // No performance data - give neutral score
            score += 10.0;
        }
        
        // System capability compatibility
        if model.size_mb as u64 > self.system_info.available_memory_mb / 2 {
            score -= 20.0;  // Penalize models that might cause memory issues
        }
        
        // Model-specific optimizations
        score += self.get_model_specific_bonus(model, criteria);
        
        debug!("Model '{}' scored {:.2} points", model.name, score);
        Ok(score.max(0.0))  // Ensure non-negative scores
    }
    
    fn get_model_specific_bonus(&self, model: &EmbeddingModel, criteria: &SelectionCriteria) -> f64 {
        let mut bonus = 0.0;
        
        // SweRankEmbed-Small specific bonuses
        if model.name == "swerank-embed-small" {
            bonus += match criteria.task_type {
                TaskType::CodeSearch => 10.0,
                TaskType::ConceptualSearch => 8.0,
                TaskType::CrossLanguageSearch => 6.0,
                _ => 4.0,
            };
        }
        
        // Nomic-embed bonuses
        if model.name == "nomic-embed-text" {
            bonus += match criteria.task_type {
                TaskType::DocumentationSearch => 8.0,
                TaskType::QuickSearch => 6.0,
                _ => 4.0,
            };
        }
        
        // MXBAI-embed bonuses  
        if model.name == "mxbai-embed-large" {
            bonus += match criteria.task_type {
                TaskType::HighAccuracySearch => 12.0,
                TaskType::LargeCodebaseSearch => 10.0,
                TaskType::ConceptualSearch => 8.0,
                _ => 4.0,
            };
        }
        
        // Size-based preferences
        if criteria.prefer_fast && model.size_mb < 200 {
            bonus += 5.0;
        }
        
        if criteria.prefer_accurate && model.size_mb > 500 {
            bonus += 5.0;
        }
        
        bonus
    }
    
    async fn verify_model_health(&mut self, manager: &mut EmbeddingManager, model: &EmbeddingModel) -> Result<()> {
        debug!("Verifying health of model: {}", model.name);
        
        let test_text = "function calculateSum(a, b) { return a + b; }";
        let start_time = std::time::Instant::now();
        
        match manager.generate_embeddings_with_model(test_text, &model.name).await {
            Ok(embeddings) => {
                let elapsed = start_time.elapsed();
                
                if embeddings.is_empty() {
                    anyhow::bail!("Model returned empty embeddings");
                }
                
                // Update performance cache
                self.update_performance_cache(&model.name, elapsed.as_millis() as f64, true);
                
                info!("Model '{}' health check passed ({:.0}ms)", model.name, elapsed.as_millis());
                Ok(())
            },
            Err(e) => {
                warn!("Model '{}' health check failed: {}", model.name, e);
                self.update_performance_cache(&model.name, 0.0, false);
                Err(e)
            }
        }
    }
    
    fn update_performance_cache(&mut self, model_name: &str, response_time_ms: f64, success: bool) {
        let perf = self.performance_cache.entry(model_name.to_string())
            .or_insert_with(|| ModelPerformance {
                model_name: model_name.to_string(),
                avg_response_time_ms: response_time_ms,
                reliability_score: if success { 1.0 } else { 0.0 },
                last_successful_use: chrono::Utc::now(),
                consecutive_failures: if success { 0 } else { 1 },
                embedding_quality_score: 0.5,  // Default neutral score
            });
        
        if success {
            // Update moving average
            perf.avg_response_time_ms = (perf.avg_response_time_ms * 0.8) + (response_time_ms * 0.2);
            perf.reliability_score = (perf.reliability_score * 0.9) + 0.1;
            perf.last_successful_use = chrono::Utc::now();
            perf.consecutive_failures = 0;
        } else {
            perf.consecutive_failures += 1;
            perf.reliability_score *= 0.8;  // Decay reliability on failure
        }
    }
    
    fn detect_system_capabilities() -> SystemCapabilities {
        let available_memory_mb = Self::get_available_memory_mb();
        let cpu_cores = num_cpus::get();
        let network_available = Self::check_network_connectivity();
        
        SystemCapabilities {
            available_memory_mb,
            cpu_cores,
            network_available,
            ollama_running: false,  // Will be updated by manager
            has_gpu: false,        // Future GPU detection
        }
    }
    
    fn get_available_memory_mb() -> u64 {
        // Conservative estimate - assume 1GB available for embeddings
        // In production, this would use platform-specific APIs
        1024
    }
    
    fn check_network_connectivity() -> bool {
        // Simple check - in production would actually test network
        true
    }
    
    fn initialize_task_preferences() -> TaskPreferences {
        let mut preferences = HashMap::new();
        
        // Code search preferences
        preferences.insert(TaskType::CodeSearch, vec![
            ModelPreference {
                model_name: "swerank-embed-small".to_string(),
                provider: "embedded".to_string(),
                preference_score: 0.95,
                min_memory_mb: 200,
                reasoning: "Specifically trained for software issue localization".to_string(),
            },
            ModelPreference {
                model_name: "nomic-embed-text".to_string(),
                provider: "ollama".to_string(),
                preference_score: 0.85,
                min_memory_mb: 400,
                reasoning: "Code-optimized, excellent for semantic code search".to_string(),
            },
            ModelPreference {
                model_name: "mxbai-embed-large".to_string(),
                provider: "ollama".to_string(),
                preference_score: 0.90,
                min_memory_mb: 800,
                reasoning: "High-quality embeddings for complex code analysis".to_string(),
            },
        ]);
        
        // Quick search preferences (speed over accuracy)
        preferences.insert(TaskType::QuickSearch, vec![
            ModelPreference {
                model_name: "swerank-embed-small".to_string(),
                provider: "embedded".to_string(),
                preference_score: 0.90,
                min_memory_mb: 200,
                reasoning: "Fast embedded model with good quality".to_string(),
            },
            ModelPreference {
                model_name: "all-MiniLM-L6-v2".to_string(),
                provider: "huggingface".to_string(),
                preference_score: 0.70,
                min_memory_mb: 100,
                reasoning: "Lightweight and fast".to_string(),
            },
        ]);
        
        // High accuracy preferences (accuracy over speed)
        preferences.insert(TaskType::HighAccuracySearch, vec![
            ModelPreference {
                model_name: "mxbai-embed-large".to_string(),
                provider: "ollama".to_string(),
                preference_score: 0.95,
                min_memory_mb: 800,
                reasoning: "Highest quality embeddings available".to_string(),
            },
            ModelPreference {
                model_name: "swerank-embed-small".to_string(),
                provider: "embedded".to_string(),
                preference_score: 0.85,
                min_memory_mb: 200,
                reasoning: "SOTA code-specific performance".to_string(),
            },
        ]);
        
        // Cross-language search
        preferences.insert(TaskType::CrossLanguageSearch, vec![
            ModelPreference {
                model_name: "bge-m3".to_string(),
                provider: "ollama".to_string(),
                preference_score: 0.90,
                min_memory_mb: 1500,
                reasoning: "Multilingual support for mixed-language codebases".to_string(),
            },
            ModelPreference {
                model_name: "mxbai-embed-large".to_string(),
                provider: "ollama".to_string(),
                preference_score: 0.85,
                min_memory_mb: 800,
                reasoning: "Strong cross-language capabilities".to_string(),
            },
        ]);
        
        TaskPreferences { preferences }
    }
    
    /// Get performance statistics for a model
    pub fn get_model_performance(&self, model_name: &str) -> Option<&ModelPerformance> {
        self.performance_cache.get(model_name)
    }
    
    /// Update system capabilities (call this periodically)
    pub async fn refresh_system_info(&mut self, manager: &mut EmbeddingManager) {
        self.system_info.ollama_running = manager.check_ollama_availability().await;
        self.system_info.network_available = Self::check_network_connectivity();
        
        debug!("Updated system info: Ollama={}, Network={}", 
               self.system_info.ollama_running, 
               self.system_info.network_available);
    }
}

impl Default for AutoSelectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_preferences_initialization() {
        let engine = AutoSelectionEngine::new();
        
        // Should have preferences for all task types
        assert!(engine.task_preferences.preferences.contains_key(&TaskType::CodeSearch));
        assert!(engine.task_preferences.preferences.contains_key(&TaskType::QuickSearch));
        assert!(engine.task_preferences.preferences.contains_key(&TaskType::HighAccuracySearch));
        
        // Code search should prefer SweRankEmbed
        let code_prefs = &engine.task_preferences.preferences[&TaskType::CodeSearch];
        let swerank_pref = code_prefs.iter().find(|p| p.model_name == "swerank-embed-small");
        assert!(swerank_pref.is_some());
        assert!(swerank_pref.unwrap().preference_score > 0.9);
    }
    
    #[test]
    fn test_system_capabilities_detection() {
        let caps = AutoSelectionEngine::detect_system_capabilities();
        
        assert!(caps.cpu_cores > 0);
        assert!(caps.available_memory_mb > 0);
    }
    
    #[test]
    fn test_selection_criteria_default() {
        let criteria = SelectionCriteria::default();
        
        assert_eq!(criteria.task_type, TaskType::CodeSearch);
        assert!(criteria.prefer_local);
        assert!(criteria.prefer_accurate);
        assert!(!criteria.prefer_fast);
    }
}