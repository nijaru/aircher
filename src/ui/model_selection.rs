use ratatui::{
    layout::Rect,
    Frame,
};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, error};

use crate::config::{ConfigManager, ModelConfig};
use crate::providers::ProviderManager;
use crate::auth::{AuthManager, AuthStatus};
use super::typeahead::{TypeaheadOverlay, TypeaheadItem};
use std::sync::Arc;

#[derive(Debug)]
enum ModelUpdate {
    ModelsReceived {
        provider: String,
        models: Vec<String>,
    },
    ModelsFetchError {
        provider: String,
        error: String,
    },
}

fn get_ollama_context_window(model_name: &str) -> u32 {
    // Extract context window from model name patterns
    let name_lower = model_name.to_lowercase();
    
    // Check for explicit context indicators
    if name_lower.contains("128k") {
        return 128000;
    } else if name_lower.contains("32k") {
        return 32768;
    } else if name_lower.contains("16k") {
        return 16384;
    } else if name_lower.contains("8k") {
        return 8192;
    } else if name_lower.contains("4k") {
        return 4096;
    } else if name_lower.contains("2k") {
        return 2048;
    }
    
    // Model-specific defaults based on common Ollama models
    if name_lower.contains("llama3.3") || name_lower.contains("llama-3.3") {
        128000
    } else if name_lower.contains("llama3.2") || name_lower.contains("llama-3.2") {
        128000
    } else if name_lower.contains("llama3.1") || name_lower.contains("llama-3.1") {
        128000
    } else if name_lower.contains("llama3") || name_lower.contains("llama-3") {
        8192
    } else if name_lower.contains("qwen2.5") {
        128000
    } else if name_lower.contains("qwen") {
        32768
    } else if name_lower.contains("deepseek-r1") {
        64000
    } else if name_lower.contains("deepseek") {
        16384
    } else if name_lower.contains("mixtral") {
        32768
    } else if name_lower.contains("mistral") {
        8192
    } else if name_lower.contains("gemma2") {
        8192
    } else if name_lower.contains("phi") {
        128000
    } else if name_lower.contains("yi") {
        200000
    } else if name_lower.contains("command-r") {
        128000
    } else {
        // Conservative default
        4096
    }
}

pub struct ModelSelectionOverlay {
    mode: SelectionMode,
    model_typeahead: TypeaheadOverlay,
    provider_typeahead: TypeaheadOverlay,
    current_provider: String,
    current_model: String,
    provider_models: HashMap<String, Vec<ModelConfig>>,
    has_providers: bool,
    auth_manager: Option<Arc<AuthManager>>,
    provider_manager: Option<Arc<ProviderManager>>,
    fetching_models: bool,
    models_fetched_for: Option<String>, // Track which provider we've fetched models for
    model_update_rx: Option<mpsc::UnboundedReceiver<ModelUpdate>>,
    model_update_tx: mpsc::UnboundedSender<ModelUpdate>,
    // Track last selected model for each provider
    last_selected_models: HashMap<String, String>,
}

#[derive(Clone, Copy, PartialEq)]
enum SelectionMode {
    Model,
    Provider,
}

impl ModelSelectionOverlay {
    pub fn new(config: &ConfigManager) -> Self {
        let model_typeahead = TypeaheadOverlay::new(
            "Select Model".to_string(),
            String::new(), // Will be updated with current model
        );
        
        let provider_typeahead = TypeaheadOverlay::new(
            "Select Provider".to_string(),
            String::new(), // Will be updated with current provider
        );

        // Get current settings
        let current_provider = config.global.default_provider.clone();
        let current_model = config.global.default_model.clone();

        // Build provider models map
        let mut provider_models = HashMap::new();
        for (name, provider_config) in &config.providers {
            // Skip loading config models for providers with dynamic model fetching
            let is_dynamic = matches!(name.as_str(), "ollama" | "openai" | "openrouter");
            if !is_dynamic {
                provider_models.insert(name.clone(), provider_config.models.clone());
            }
        }

        // Create channel for model updates
        let (model_update_tx, model_update_rx) = mpsc::unbounded_channel();

        let mut overlay = Self {
            mode: SelectionMode::Model,
            model_typeahead,
            provider_typeahead,
            current_provider: current_provider.clone(),
            current_model: current_model.clone(),
            provider_models,
            has_providers: false,
            auth_manager: None,
            provider_manager: None,
            fetching_models: false,
            models_fetched_for: None,
            model_update_rx: Some(model_update_rx),
            model_update_tx,
            last_selected_models: HashMap::new(),
        };
        
        // Hide input for provider selection
        overlay.provider_typeahead.hide_input = true;

        overlay.update_items(config);
        overlay
    }

    pub fn with_providers(config: &ConfigManager, providers: &ProviderManager) -> Self {
        let mut overlay = Self::new(config);
        overlay.has_providers = true;
        overlay.update_provider_availability(providers);
        // Update models with dynamic data from providers
        overlay.update_dynamic_models(providers);
        overlay
    }

    pub fn with_auth_manager(config: &ConfigManager, auth_manager: Arc<AuthManager>) -> Self {
        let mut overlay = Self::new(config);
        overlay.auth_manager = Some(auth_manager);
        overlay.update_items(config);
        overlay
    }

    pub fn set_provider_manager(&mut self, provider_manager: Arc<ProviderManager>) {
        debug!("✅ set_provider_manager called - provider_manager is now available");
        self.provider_manager = Some(provider_manager);
        
        // If we're currently showing a loading state due to missing provider manager, refresh models
        if self.model_typeahead.items.len() == 1 {
            if let Some(item) = self.model_typeahead.items.first() {
                if item.value == "_loading" {
                    debug!("⚡ Provider manager set, refreshing models to replace loading state");
                    self.update_model_items();
                }
            }
        }
    }

    fn update_items(&mut self, config: &ConfigManager) {
        // Use async auth checking if auth manager is available
        if self.auth_manager.is_some() {
            // We can't make this async directly, but we can schedule the async update
            // For initial display, show providers with temporary status
            self.update_items_with_temp_status(config);
            return;
        }
        
        // Fall back to legacy method when no auth manager is available
        if self.auth_manager.is_none() {
            let mut provider_legacy_items: Vec<_> = config.providers
                .iter()
                .map(|(name, provider_config)| {
                    let (is_local, is_authenticated) = if provider_config.api_key_env.is_empty() {
                        // Empty api_key_env could be local (Ollama) or OAuth (anthropic-pro)
                        if name == "ollama" {
                            (true, true) // Local provider - always available
                        } else {
                            (false, false) // OAuth provider - not authenticated by default
                        }
                    } else {
                        (false, std::env::var(&provider_config.api_key_env).is_ok()) // Remote provider - check auth
                    };
                    (name.clone(), is_local, is_authenticated, provider_config)
                })
                .collect();
                
            // Sort providers: authenticated (including local) first, then unauthenticated, alphabetical within each group
            provider_legacy_items.sort_by(|(name_a, _, auth_a, _), (name_b, _, auth_b, _)| {
                match (auth_a, auth_b) {
                    (true, true) => name_a.cmp(name_b),
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    (false, false) => name_a.cmp(name_b),
                }
            });
            
            let provider_items: Vec<TypeaheadItem> = provider_legacy_items
                .into_iter()
                .map(|(name, _is_local, is_authenticated, _provider_config)| TypeaheadItem {
                    label: format_provider_name_with_status_legacy(&name, config),
                    value: name.clone(),
                    description: Some(format_provider_description_with_auth_legacy(&name, config)),
                    available: is_authenticated,
                })
                .collect();

            self.provider_typeahead.set_items(provider_items);
            self.provider_typeahead.set_current_value(Some(self.current_provider.clone()));
            self.update_provider_description();

            // Update model items for current provider
            self.update_model_items();
            return;
        }

    }
    
    fn update_items_with_temp_status(&mut self, config: &ConfigManager) {
        // Show all providers with "checking..." status initially
        let mut provider_items: Vec<_> = config.providers
            .keys()
            .map(|name| {
                TypeaheadItem {
                    label: format!("◦ {} (checking...)", format_provider_name(name)),
                    value: name.clone(),
                    description: Some("Verifying authentication status...".to_string()),
                    available: false, // Temporarily unavailable until checked
                }
            })
            .collect();
            
        // Sort alphabetically for initial display
        provider_items.sort_by(|a, b| a.value.cmp(&b.value));
        
        self.provider_typeahead.set_items(provider_items);
        self.provider_typeahead.set_current_value(Some(self.current_provider.clone()));
        self.update_provider_description();
        
        // Note: The actual auth status will be updated when the parent calls initialize_auth_status()
    }

    fn update_items_legacy(&mut self, config: &ConfigManager) {
        let mut provider_legacy_items: Vec<_> = config.providers
            .iter()
            .map(|(name, provider_config)| {
                let (is_local, is_authenticated) = if provider_config.api_key_env.is_empty() {
                    // Empty api_key_env could be local (Ollama) or OAuth (anthropic-pro)
                    if name == "ollama" {
                        (true, true) // Local provider - always available
                    } else {
                        (false, false) // OAuth provider - not authenticated by default
                    }
                } else {
                    (false, std::env::var(&provider_config.api_key_env).is_ok()) // Remote provider - check auth
                };
                (name.clone(), is_local, is_authenticated, provider_config)
            })
            .collect();
            
        // Sort providers: authenticated (including local) first, then unauthenticated, alphabetical within each group
        provider_legacy_items.sort_by(|(name_a, _, auth_a, _), (name_b, _, auth_b, _)| {
            match (auth_a, auth_b) {
                (true, true) => name_a.cmp(name_b),
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                (false, false) => name_a.cmp(name_b),
            }
        });
        
        let provider_items: Vec<TypeaheadItem> = provider_legacy_items
            .into_iter()
            .map(|(name, _is_local, is_authenticated, _provider_config)| TypeaheadItem {
                label: format_provider_name_with_status_legacy(&name, config),
                value: name.clone(),
                description: Some(format_provider_description_with_auth_legacy(&name, config)),
                available: is_authenticated,
            })
            .collect();

        self.provider_typeahead.set_items(provider_items);
        self.provider_typeahead.set_current_value(Some(self.current_provider.clone()));
        self.update_provider_description();

        // Update model items for current provider
        self.update_model_items();
    }

    pub async fn update_items_with_auth(&mut self, config: &ConfigManager) {
        debug!("update_items_with_auth called");
        if let Some(auth_manager) = &self.auth_manager {
            let auth_statuses = auth_manager.get_all_provider_statuses(config).await;
            debug!("Got auth statuses for {} providers", auth_statuses.len());
            
            let mut provider_items: Vec<_> = config.providers
                .iter()
                .map(|(name, provider_config)| {
                    let auth_status = auth_statuses.get(name)
                        .map(|info| &info.status)
                        .unwrap_or(&AuthStatus::NotConfigured);
                    
                    debug!("Provider '{}' auth status: {:?}", name, auth_status);
                    
                    (name.clone(), auth_status.clone(), provider_config)
                })
                .collect();
                
            // Sort providers: authenticated first (alphabetical), then unauthenticated (alphabetical)
            provider_items.sort_by(|(name_a, auth_a, _), (name_b, auth_b, _)| {
                match (auth_a, auth_b) {
                    // Both authenticated - sort alphabetically
                    (AuthStatus::Authenticated, AuthStatus::Authenticated) => name_a.cmp(name_b),
                    // Authenticated comes before unauthenticated
                    (AuthStatus::Authenticated, _) => std::cmp::Ordering::Less,
                    (_, AuthStatus::Authenticated) => std::cmp::Ordering::Greater,
                    // Both unauthenticated - sort alphabetically
                    _ => name_a.cmp(name_b),
                }
            });
            
            let provider_typeahead_items: Vec<TypeaheadItem> = provider_items
                .into_iter()
                .map(|(name, auth_status, _provider_config)| {
                    TypeaheadItem {
                        label: format_provider_name_with_auth_status(&name, &auth_status),
                        value: name.clone(),
                        description: Some(format_provider_description_with_auth_status(&name, &auth_status, config)),
                        available: {
                            let is_auth = matches!(auth_status, AuthStatus::Authenticated);
                            debug!("Provider {} auth status: {:?}, available: {}", name, auth_status, is_auth);
                            // Special case: if update_provider_availability was called and marked this as available,
                            // preserve that status (important for Ollama which is always available locally)
                            is_auth
                        },
                    }
                })
                .collect();

            self.provider_typeahead.set_items(provider_typeahead_items);
            self.provider_typeahead.set_current_value(Some(self.current_provider.clone()));
            self.update_provider_description();

            // Update model items for current provider
            self.update_model_items();
        } else {
            self.update_items_legacy(config);
        }
    }

    pub fn update_provider_availability(&mut self, providers: &ProviderManager) {
        // Update availability based on actual provider manager
        let provider_list = providers.list_all();
        debug!("update_provider_availability: Available providers: {:?}", provider_list);
        
        let mut any_changed = false;
        if let Some(items) = self.provider_typeahead.items.clone().into_iter()
            .map(|mut item| {
                let was_available = item.available;
                item.available = provider_list.contains(&item.value);
                if was_available != item.available {
                    debug!("Provider {} availability changed from {} to {}", item.value, was_available, item.available);
                    any_changed = true;
                }
                Some(item)
            })
            .collect::<Option<Vec<_>>>() {
            self.provider_typeahead.set_items(items);
            
            // If availability changed and we're in model selection mode, refresh the model items
            if any_changed && self.mode == SelectionMode::Model {
                debug!("Provider availability changed, refreshing model items");
                self.update_model_items();
            }
        }
    }

    /// Update model lists with dynamic data from providers
    pub fn update_dynamic_models(&mut self, providers: &ProviderManager) {
        // For Ollama, get dynamic model list instead of using config
        let ollama_models = providers.get_provider_models("ollama");
        debug!("update_dynamic_models: Got {} Ollama models from provider manager", ollama_models.len());
        if !ollama_models.is_empty() {
            // Convert to ModelConfig format for consistency
            let dynamic_models: Vec<crate::config::ModelConfig> = ollama_models.into_iter()
                .map(|model_name| crate::config::ModelConfig {
                    name: model_name,
                    context_window: 4096, // Default for Ollama
                    input_cost_per_1m: 0.0, // Free
                    output_cost_per_1m: 0.0, // Free
                    supports_streaming: true, // Ollama supports streaming
                    supports_tools: false,
                })
                .collect();
            
            // Update the provider_models map for Ollama
            self.provider_models.insert("ollama".to_string(), dynamic_models);
            
            // If current provider is Ollama, refresh the model items
            if self.current_provider == "ollama" {
                self.update_model_items();
            }
        }
    }

    /// Initialize auth status for the overlay - call this after construction
    pub async fn initialize_auth_status(&mut self, config: &ConfigManager) {
        if self.auth_manager.is_some() {
            self.update_items_with_auth(config).await;
        }
    }

    fn update_model_items(&mut self) {
        debug!("=== UPDATE_MODEL_ITEMS CALLED for provider: {} ===", self.current_provider);
        debug!("Provider manager available: {}", self.provider_manager.is_some());
        
        // Check if provider is authenticated first
        let is_authenticated = self.provider_typeahead.items.iter()
            .find(|item| item.value == self.current_provider)
            .map(|item| item.available)
            .unwrap_or(false);

        debug!("update_model_items: provider={}, is_authenticated={}, provider_items_count={}", 
               self.current_provider, is_authenticated, self.provider_typeahead.items.len());
        
        // Debug: Print all provider items to see their availability status
        for (idx, item) in self.provider_typeahead.items.iter().enumerate() {
            debug!("Provider item {}: value='{}', available={}, label='{}'", 
                   idx, item.value, item.available, item.label);
        }
        

        if !is_authenticated {
            debug!("AUTHENTICATION FAILED - creating error message for {}", self.current_provider);
            // Show provider-specific error messages
            let (label, description) = match self.current_provider.as_str() {
                "ollama" => {
                    // Check if OLLAMA_HOST is set to provide specific guidance
                    let ollama_host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "localhost".to_string());
                    let host_display = if ollama_host == "localhost" { 
                        "localhost:11434".to_string() 
                    } else { 
                        format!("{}:11434", ollama_host)
                    };
                    
                    (
                        "❌ Ollama not available".to_string(),
                        Some(format!(
                            "Cannot connect to Ollama at {}. {}",
                            host_display,
                            if ollama_host == "localhost" {
                                "Start Ollama with 'ollama serve' or set OLLAMA_HOST env var"
                            } else {
                                "Check if Ollama is running and accessible"
                            }
                        ))
                    )
                },
                "openai" | "openrouter" => (
                    "❌ API key required".to_string(),
                    Some("Run /auth to configure API key".to_string())
                ),
                "anthropic-api" => (
                    "❌ API key required".to_string(),
                    Some("Set ANTHROPIC_API_KEY environment variable or run /auth".to_string())
                ),
                "anthropic-pro" => (
                    "❌ Authentication required".to_string(),
                    Some("Run /auth to authenticate with Claude Pro".to_string())
                ),
                _ => (
                    "❌ Authentication required".to_string(),
                    Some("Run /auth to set up authentication".to_string())
                )
            };
            
            let no_auth_item = TypeaheadItem {
                label,
                value: "_no_auth".to_string(),
                description,
                available: false,
            };
            debug!("Setting error message item: label='{}', description='{:?}'", 
                   no_auth_item.label, no_auth_item.description);
            self.model_typeahead.set_items(vec![no_auth_item]);
            self.model_typeahead.set_current_value(None);
            self.update_model_description();
            debug!("ERROR MESSAGE SET - returning from update_model_items");
            return;
        }

        // For providers that support dynamic model fetching (like Ollama), 
        // don't show config models - wait for real ones
        // BUT only if the provider is authenticated - unauthenticated providers should show error messages
        let is_dynamic_provider = matches!(self.current_provider.as_str(), "ollama" | "openai" | "openrouter");
        
        debug!("update_model_items: provider={}, is_dynamic={}, is_authenticated={}, models_fetched_for={:?}, fetching={}", 
            self.current_provider, is_dynamic_provider, is_authenticated, self.models_fetched_for, self.fetching_models);
        
        // If this is a dynamic provider AND authenticated and we haven't fetched models yet, show loading
        if is_dynamic_provider && is_authenticated && !self.models_fetched_for.as_ref().map_or(false, |p| p == &self.current_provider) {
            // Check if provider_manager is available
            if self.provider_manager.is_none() {
                debug!("⏳ Provider manager not yet available, showing loading state for {}", self.current_provider);
                let loading_item = TypeaheadItem {
                    label: "🔄 Loading models...".to_string(),
                    value: "_loading".to_string(),
                    description: Some("Initializing provider connection".to_string()),
                    available: false,
                };
                self.model_typeahead.set_items(vec![loading_item]);
                self.model_typeahead.set_current_value(None);
                self.update_model_description();
                // Don't return - we want to continue and show config models if available
                debug!("⏳ Provider manager not available, will retry when provider manager is set");
                return;
            }
            
            // Trigger fetch if not already fetching
            if !self.fetching_models {
                debug!("AUTHENTICATED DYNAMIC PROVIDER: Triggering fetch for {}", self.current_provider);
                self.fetch_models_for_provider(&self.current_provider.clone());
            } else {
                debug!("Already fetching models for {}", self.current_provider);
            }
            // Make sure description is updated even when loading
            self.update_model_description();
            debug!("RETURNING EARLY for dynamic provider fetch");
            return; // The loading state is set by fetch_models_for_provider
        }

        let models = self.provider_models.get(&self.current_provider)
            .cloned()
            .unwrap_or_default();

        debug!("update_model_items: Found {} models for provider {}", models.len(), self.current_provider);
        let mut model_items: Vec<TypeaheadItem> = Vec::new();
        
        // Add "Default" option for Anthropic providers (similar to Claude Code)
        if self.current_provider == "anthropic-api" || self.current_provider == "anthropic-pro" {
            model_items.push(TypeaheadItem {
                label: "Default (Smart model selection based on usage)".to_string(),
                value: "default".to_string(),
                description: Some("Automatically selects the best model based on task complexity and usage limits".to_string()),
                available: true,
            });
        }
        
        // Add regular models
        model_items.extend(models.into_iter()
            .enumerate()
            .map(|(idx, model)| {
                // Format label with visual indicators
                let mut label = model.name.clone();
                
                // Mark the first model as default with a star
                if idx == 0 {
                    label = format!("⭐ {}", label);
                }
                
                // Add visual size indicator for very large models
                if model.context_window >= 1_000_000 {
                    label = format!("{} 🧠", label); // Brain for large context
                }
                
                TypeaheadItem {
                    label,
                    value: model.name.clone(),
                    description: format_model_description(&model),
                    available: true, // Models are available if provider is configured
                }
            }));

        // If no models available, show provider-specific message
        if model_items.is_empty() {
            debug!("No models available for {}, showing empty state", self.current_provider);
            
            let (label, description) = match self.current_provider.as_str() {
                "ollama" => {
                    // For Ollama, this means the provider was marked as available but model fetching failed
                    // This usually indicates a connection issue that happened after initial auth check
                    let ollama_host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "localhost".to_string());
                    let host_display = if ollama_host == "localhost" { 
                        "localhost:11434".to_string() 
                    } else { 
                        format!("{}:11434", ollama_host)
                    };
                    
                    (
                        "❌ Ollama connection failed".to_string(),
                        Some(format!(
                            "Could not fetch models from Ollama at {}. {}",
                            host_display,
                            if ollama_host == "localhost" {
                                "Check if Ollama is running with 'ollama serve'"
                            } else {
                                "Verify Ollama is running and network is accessible"
                            }
                        ))
                    )
                },
                "openai" | "openrouter" => (
                    "❌ No models found".to_string(),
                    Some("API key may be invalid or no models accessible".to_string())
                ),
                _ => (
                    "No models available".to_string(),
                    Some("Check provider configuration or try refreshing".to_string())
                )
            };
            
            model_items.push(TypeaheadItem {
                label,
                value: "_empty".to_string(),
                description,
                available: false,
            });
        }
        
        debug!("Final model_items count: {}, items: {:?}", model_items.len(), 
               model_items.iter().map(|item| &item.label).collect::<Vec<_>>());
        
        self.model_typeahead.set_items(model_items);
        self.model_typeahead.set_current_value(Some(self.current_model.clone()));
        self.update_model_description();
        
        debug!("After set_items, typeahead filtered_items count: {}", 
               self.model_typeahead.filtered_items.len());
    }

    fn update_provider_description(&mut self) {
        self.provider_typeahead.description = format!(
            "Current provider: {}\n\n[ Tab ] or [←/→] Switch to model selection", 
            format_provider_name(&self.current_provider)
        );
    }

    fn update_model_description(&mut self) {
        // Only show current model if it's a valid selection for this provider
        let has_valid_model = self.provider_models.get(&self.current_provider)
            .map(|models| models.iter().any(|m| m.name == self.current_model))
            .unwrap_or(false);
        
        let model_text = if has_valid_model {
            format!("Current model: {}", &self.current_model)
        } else {
            "Select a model".to_string()
        };
        
        self.model_typeahead.description = format!(
            "{}\n\n[ Tab ] or [←/→] Switch to provider selection",
            model_text
        );
    }

    pub fn show(&mut self) {
        // Smart default: check if current provider is authenticated
        let should_start_with_provider = !self.is_current_provider_authenticated();
        
        if should_start_with_provider && self.mode == SelectionMode::Model {
            // Switch to provider selection if current provider not authenticated
            self.mode = SelectionMode::Provider;
        }
        
        // For model selection mode, ensure models are loaded/loading
        if self.mode == SelectionMode::Model {
            debug!("show() - about to call update_model_items, provider_manager available: {}", self.provider_manager.is_some());
            // Update model items which will trigger fetching if needed
            self.update_model_items();
        }
        
        match self.mode {
            SelectionMode::Model => self.model_typeahead.show(),
            SelectionMode::Provider => self.provider_typeahead.show(),
        }
    }
    
    /// Ensure provider manager is set - call this before any model operations
    pub fn ensure_provider_manager(&mut self, provider_manager: Arc<ProviderManager>) {
        if self.provider_manager.is_none() {
            debug!("🔧 ensure_provider_manager: Setting provider_manager that was missing");
            self.provider_manager = Some(provider_manager);
        }
    }
    

    fn fetch_models_for_provider(&mut self, provider_name: &str) {
        debug!("=== FETCH_MODELS_FOR_PROVIDER called for: {} ===", provider_name);
        // Don't fetch if already fetching or no provider manager
        if self.fetching_models || self.provider_manager.is_none() {
            debug!("Cannot fetch: fetching={}, has_manager={}", self.fetching_models, self.provider_manager.is_some());
            return;
        }

        // Don't fetch if provider not authenticated
        let is_authenticated = self.provider_typeahead.items.iter()
            .find(|item| item.value == provider_name)
            .map(|item| item.available)
            .unwrap_or(false);

        if !is_authenticated {
            debug!("Provider {} is not authenticated, skipping model fetch", provider_name);
            return;
        }

        self.fetching_models = true;
        self.models_fetched_for = Some(provider_name.to_string());
        
        // Update model typeahead to show loading state
        let loading_item = TypeaheadItem {
            label: "⏳ Loading models...".to_string(),
            value: "_loading".to_string(),
            description: Some("Fetching available models from provider".to_string()),
            available: false,
        };
        debug!("Setting loading state for provider: {}", provider_name);
        self.model_typeahead.set_items(vec![loading_item]);
        self.model_typeahead.set_current_value(None); // Clear current value to ensure loading shows
        self.model_typeahead.filter_items(); // Ensure the list updates
        self.update_model_description();

        // Spawn async task to fetch models
        if let Some(provider_manager) = self.provider_manager.clone() {
            let provider_name = provider_name.to_string();
            let tx = self.model_update_tx.clone();
            
            debug!("Spawning async task to fetch models for {}", provider_name);
            tokio::spawn(async move {
                debug!("=== ASYNC TASK STARTED for {} ===", provider_name);
                match provider_manager.get_provider_or_host(&provider_name) {
                    Some(provider) => {
                        debug!("Got provider for {}, calling list_available_models", provider_name);
                        match provider.list_available_models().await {
                            Ok(models) => {
                                debug!("*** SUCCESSFULLY FETCHED {} models for {}: {:?} ***", 
                                       models.len(), provider_name, models);
                                if let Err(e) = tx.send(ModelUpdate::ModelsReceived {
                                    provider: provider_name,
                                    models,
                                }) {
                                    error!("Failed to send model update: {}", e);
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(ModelUpdate::ModelsFetchError {
                                    provider: provider_name,
                                    error: e.to_string(),
                                });
                            }
                        }
                    }
                    None => {
                        let _ = tx.send(ModelUpdate::ModelsFetchError {
                            provider: provider_name,
                            error: "Provider not found".to_string(),
                        });
                    }
                }
            });
        } else {
            debug!("No provider_manager available for fetching models");
        }
    }

    /// Process any pending model updates from the async channel
    pub fn process_model_updates(&mut self) {
        // Collect updates first to avoid borrowing issues
        let mut updates = Vec::new();
        if let Some(rx) = &mut self.model_update_rx {
            while let Ok(update) = rx.try_recv() {
                updates.push(update);
            }
        }
        
        if !updates.is_empty() {
            debug!("process_model_updates: Processing {} updates", updates.len());
        }
        
        // Process collected updates
        for update in updates {
            match update {
                ModelUpdate::ModelsReceived { provider, models } => {
                    debug!("Received {} models for provider {}", models.len(), provider);
                    // Only process if this is for the current fetch
                    if self.models_fetched_for.as_ref() == Some(&provider) {
                        self.fetching_models = false;
                        
                        // Check if provider is still authenticated before processing update
                        let is_provider_authenticated = self.provider_typeahead.items.iter()
                            .find(|item| item.value == provider)
                            .map(|item| item.available)
                            .unwrap_or(false);
                            
                        if !is_provider_authenticated {
                            debug!("Ignoring models received for unauthenticated provider: {}", provider);
                            return;
                        }
                        
                        // Convert to ModelConfig format
                        let model_configs: Vec<ModelConfig> = models.into_iter()
                            .map(|name| {
                                // Try to find existing config, otherwise create default
                                self.provider_models.get(&provider)
                                    .and_then(|configs| configs.iter().find(|c| c.name == name))
                                    .cloned()
                                    .unwrap_or_else(|| {
                                        // Determine context window based on model name for Ollama
                                        let context_window = if provider == "ollama" {
                                            get_ollama_context_window(&name)
                                        } else {
                                            128000 // Default for other providers
                                        };
                                        
                                        ModelConfig {
                                            name: name.clone(),
                                            context_window,
                                            input_cost_per_1m: 0.0,
                                            output_cost_per_1m: 0.0,
                                            supports_streaming: true,
                                            supports_tools: false,
                                        }
                                    })
                            })
                            .collect();
                        
                        // Update provider models with fetched data
                        debug!("Inserting {} models for provider {}", model_configs.len(), provider);
                        self.provider_models.insert(provider.clone(), model_configs);
                        
                        // Refresh model items if this is still the current provider
                        if self.current_provider == provider {
                            debug!("Current provider matches, updating model items");
                            self.update_model_items();
                        } else {
                            debug!("Current provider {} doesn't match {}, not updating", self.current_provider, provider);
                        }
                    }
                }
                ModelUpdate::ModelsFetchError { provider, error } => {
                    if self.models_fetched_for.as_ref() == Some(&provider) {
                        self.fetching_models = false;
                        
                        // Check if provider is still authenticated before showing fetch error
                        let is_provider_authenticated = self.provider_typeahead.items.iter()
                            .find(|item| item.value == provider)
                            .map(|item| item.available)
                            .unwrap_or(false);
                            
                        if !is_provider_authenticated {
                            debug!("Ignoring fetch error for unauthenticated provider: {}", provider);
                            return;
                        }
                        
                        // Show error state
                        let error_item = TypeaheadItem {
                            label: "Failed to load models".to_string(),
                            value: "_error".to_string(),
                            description: Some(format!("Error: {}", error)),
                            available: false,
                        };
                        self.model_typeahead.set_items(vec![error_item]);
                        self.update_model_description();
                    }
                }
            }
        }
    }

    pub fn hide(&mut self) {
        self.model_typeahead.hide();
        self.provider_typeahead.hide();
    }

    pub fn is_visible(&self) -> bool {
        self.model_typeahead.visible || self.provider_typeahead.visible
    }

    pub fn switch_mode(&mut self) {
        match self.mode {
            SelectionMode::Model => {
                self.model_typeahead.hide();
                self.mode = SelectionMode::Provider;
                self.provider_typeahead.show();
            }
            SelectionMode::Provider => {
                self.provider_typeahead.hide();
                self.mode = SelectionMode::Model;
                self.model_typeahead.show();
            }
        }
    }

    pub fn handle_char(&mut self, c: char) {
        match self.mode {
            SelectionMode::Model => self.model_typeahead.insert_char(c),
            SelectionMode::Provider => self.provider_typeahead.insert_char(c),
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.mode {
            SelectionMode::Model => self.model_typeahead.delete_char(),
            SelectionMode::Provider => self.provider_typeahead.delete_char(),
        }
    }

    pub fn move_cursor_left(&mut self) {
        match self.mode {
            SelectionMode::Model => self.model_typeahead.move_cursor_left(),
            SelectionMode::Provider => self.provider_typeahead.move_cursor_left(),
        }
    }

    pub fn move_cursor_right(&mut self) {
        match self.mode {
            SelectionMode::Model => self.model_typeahead.move_cursor_right(),
            SelectionMode::Provider => self.provider_typeahead.move_cursor_right(),
        }
    }

    pub fn move_selection_down(&mut self) {
        match self.mode {
            SelectionMode::Model => self.model_typeahead.move_selection_down(),
            SelectionMode::Provider => {
                self.provider_typeahead.move_selection_down();
                // Prefetch models for the currently highlighted provider
                self.prefetch_models_for_current_selection();
            }
        }
    }

    pub fn move_selection_up(&mut self) {
        match self.mode {
            SelectionMode::Model => self.model_typeahead.move_selection_up(),
            SelectionMode::Provider => {
                self.provider_typeahead.move_selection_up();
                // Prefetch models for the currently highlighted provider
                self.prefetch_models_for_current_selection();
            }
        }
    }

    fn prefetch_models_for_current_selection(&mut self) {
        if self.mode != SelectionMode::Provider {
            return;
        }

        // Get the currently highlighted provider - clone the value to avoid borrowing issues
        let provider_to_fetch = if let Some(highlighted_item) = self.provider_typeahead.get_selected() {
            // Only prefetch if provider is authenticated and we haven't fetched models for it yet
            if highlighted_item.available && 
               self.models_fetched_for.as_ref() != Some(&highlighted_item.value) &&
               !self.fetching_models {
                Some(highlighted_item.value.clone())
            } else {
                None
            }
        } else {
            None
        };

        if let Some(provider_name) = provider_to_fetch {
            self.fetch_models_for_provider(&provider_name);
        }
    }

    pub fn get_selected(&mut self) -> Option<(String, String)> {
        match self.mode {
            SelectionMode::Model => {
                if let Some(item) = self.model_typeahead.get_selected() {
                    // Don't return selection if it's the no-auth placeholder
                    if item.value == "_no_auth" {
                        None
                    } else {
                        // Track the last selected model for this provider
                        self.last_selected_models.insert(self.current_provider.clone(), item.value.clone());
                        Some((self.current_provider.clone(), item.value.clone()))
                    }
                } else {
                    None
                }
            }
            SelectionMode::Provider => {
                if let Some(item) = self.provider_typeahead.get_selected() {
                    // When provider is selected, fetch models and switch to model selection
                    let provider_name = item.value.clone();
                    self.current_provider = provider_name.clone();
                    self.fetch_models_for_provider(&provider_name);
                    self.switch_mode();
                    
                    // After switching to model mode, restore last selected model for this provider
                    self.restore_last_selected_model();
                    
                    None // Don't return selection yet, let user pick model
                } else {
                    None
                }
            }
        }
    }
    
    /// Check if the currently selected provider is authenticated
    pub fn is_current_provider_authenticated(&self) -> bool {
        let result = self.provider_typeahead.items.iter()
            .find(|item| item.value == self.current_provider)
            .map(|item| {
                debug!("Provider '{}' available: {}, label: '{}'", 
                    item.value, item.available, item.label);
                item.available
            })
            .unwrap_or(false);
        debug!("is_current_provider_authenticated for '{}': {}", self.current_provider, result);
        result
    }
    
    /// Get the current provider name (for auth setup)
    pub fn get_current_provider(&self) -> &str {
        &self.current_provider
    }
    
    /// Check if we're in provider selection mode
    pub fn is_in_provider_mode(&self) -> bool {
        matches!(self.mode, SelectionMode::Provider)
    }
    
    /// Handle provider selection - update current provider and switch to model mode
    pub fn handle_provider_selection(&mut self) {
        debug!("=== HANDLE_PROVIDER_SELECTION CALLED ===");
        if let Some(item) = self.provider_typeahead.get_selected() {
            let provider_value = item.value.clone();
            debug!("handle_provider_selection: Selected provider {}, available: {}", provider_value, item.available);
            debug!("handle_provider_selection: Has provider_manager: {}", self.provider_manager.is_some());
            self.current_provider = provider_value.clone();
            self.mode = SelectionMode::Model;
            self.provider_typeahead.hide();
            
            // Update model items for new provider (this will show loading state for dynamic providers)
            debug!("handle_provider_selection: About to call update_model_items");
            self.update_model_items();
            
            // Show the model selection after updating items
            debug!("handle_provider_selection: About to show model_typeahead, current items: {}", self.model_typeahead.items.len());
            self.model_typeahead.show();
            debug!("handle_provider_selection: Model typeahead shown, final items count: {}, filtered count: {}", 
                   self.model_typeahead.items.len(), self.model_typeahead.filtered_items.len());
            debug!("handle_provider_selection: Model items: {:?}", 
                   self.model_typeahead.items.iter().map(|i| &i.label).collect::<Vec<_>>());
        } else {
            debug!("handle_provider_selection: No item selected");
        }
    }

    fn restore_last_selected_model(&mut self) {
        if let Some(last_model) = self.last_selected_models.get(&self.current_provider).cloned() {
            // Set the current value to the last selected model for this provider
            self.model_typeahead.set_current_value(Some(last_model));
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        match self.mode {
            SelectionMode::Model => self.model_typeahead.render(f, area),
            SelectionMode::Provider => self.provider_typeahead.render(f, area),
        }
    }
}

fn format_provider_name(provider: &str) -> String {
    match provider {
        "anthropic-api" => "Anthropic API".to_string(),
        "anthropic-pro" => "Anthropic Claude Pro/Max".to_string(),
        "google-gemini" => "Google Gemini API".to_string(),
        "google-vertex" => "Google Vertex AI".to_string(),
        "ollama" => "Ollama".to_string(),
        "openai" => "OpenAI".to_string(),
        "openrouter" => "OpenRouter".to_string(),
        _ => provider.to_string(),
    }
}

fn format_provider_name_with_status_legacy(provider: &str, config: &ConfigManager) -> String {
    let base_name = format_provider_name(provider);
    let status_icon = get_provider_auth_status_icon_legacy(provider, config);
    format!("{} {}", status_icon, base_name)
}

fn format_provider_description_with_auth_legacy(provider: &str, config: &ConfigManager) -> String {
    let base_description = format_provider_description(provider);
    let auth_status = get_provider_auth_description_legacy(provider, config);
    let model_count = config.providers.get(provider)
        .map(|p| p.models.len())
        .unwrap_or(0);
    
    format!("{} • {} models • {}", base_description, model_count, auth_status)
}

fn format_provider_name_with_auth_status(provider: &str, auth_status: &AuthStatus) -> String {
    let base_name = format_provider_name(provider);
    let status_icon = get_auth_status_icon_for_provider(provider, auth_status);
    format!("{} {}", status_icon, base_name)
}

fn get_auth_status_icon_for_provider(provider: &str, auth_status: &AuthStatus) -> &'static str {
    // Special handling for local providers
    if provider == "ollama" {
        match auth_status {
            AuthStatus::Authenticated => "⚡", // Local provider available
            AuthStatus::NetworkError => "✗", // Local provider not found
            _ => get_auth_status_icon(auth_status),
        }
    } else {
        get_auth_status_icon(auth_status)
    }
}

fn format_provider_description_with_auth_status(provider: &str, auth_status: &AuthStatus, config: &ConfigManager) -> String {
    let base_description = format_provider_description(provider);
    let auth_description = get_auth_status_description(auth_status);
    let model_count = config.providers.get(provider)
        .map(|p| p.models.len())
        .unwrap_or(0);
    
    format!("{} • {} models • {}", base_description, model_count, auth_description)
}

fn get_provider_auth_status_icon_legacy(provider: &str, config: &ConfigManager) -> &'static str {
    if let Some(provider_config) = config.get_provider(provider) {
        if provider_config.api_key_env.is_empty() {
            // Empty api_key_env could be local (Ollama) or OAuth (anthropic-pro)
            if provider == "ollama" {
                "⚡" // Local provider (no auth needed)
            } else {
                "✗" // OAuth provider - not authenticated by default
            }
        } else if std::env::var(&provider_config.api_key_env).is_ok() {
            "✓" // Authenticated
        } else {
            "✗" // Needs setup
        }
    } else {
        "○" // Not configured
    }
}

fn get_provider_auth_description_legacy(provider: &str, config: &ConfigManager) -> String {
    if let Some(provider_config) = config.get_provider(provider) {
        if provider_config.api_key_env.is_empty() {
            // Empty api_key_env could be local (Ollama) or OAuth (anthropic-pro)
            if provider == "ollama" {
                "local (no auth needed)".to_string()
            } else {
                "needs setup".to_string() // OAuth provider - not authenticated by default
            }
        } else if std::env::var(&provider_config.api_key_env).is_ok() {
            "authenticated".to_string()
        } else {
            "needs setup".to_string()
        }
    } else {
        "not configured".to_string()
    }
}

fn get_auth_status_icon(auth_status: &AuthStatus) -> &'static str {
    match auth_status {
        AuthStatus::Authenticated => "✓",
        AuthStatus::NotConfigured => "✗",
        AuthStatus::Invalid => "✗",
        AuthStatus::Expired => "⚠",
        AuthStatus::RateLimited => "⚠",
        AuthStatus::NetworkError => "⚠",
    }
}

fn get_auth_status_description(auth_status: &AuthStatus) -> &'static str {
    match auth_status {
        AuthStatus::Authenticated => "authenticated",
        AuthStatus::NotConfigured => "needs setup",
        AuthStatus::Invalid => "invalid key",
        AuthStatus::Expired => "expired key",
        AuthStatus::RateLimited => "rate limited",
        AuthStatus::NetworkError => "not found",
    }
}

fn format_provider_description(provider: &str) -> String {
    match provider {
        "anthropic-api" => "Pay-per-use API access with API keys".to_string(),
        "anthropic-pro" => "Subscription access with OAuth authentication".to_string(),
        "google-gemini" => "Direct Google AI API access".to_string(),
        "google-vertex" => "Enterprise Google Cloud AI platform".to_string(),
        "ollama" => "Local models running on your machine".to_string(),
        "openai" => "OpenAI API with GPT and reasoning models".to_string(),
        "openrouter" => "Multi-provider gateway with unified API".to_string(),
        _ => String::new(),
    }
}

fn format_model_description(model: &ModelConfig) -> Option<String> {
    let mut parts = Vec::new();
    
    // Context window with better formatting
    if model.context_window > 0 {
        let context_str = if model.context_window >= 1_000_000 {
            format!("{}M", model.context_window / 1_000_000)
        } else {
            format!("{}k", model.context_window / 1000)
        };
        parts.push(format!("{} ctx", context_str));
    }
    
    // Pricing with smart formatting
    if model.input_cost_per_1m > 0.0 || model.output_cost_per_1m > 0.0 {
        if model.input_cost_per_1m == model.output_cost_per_1m {
            // Same price for input/output
            parts.push(format!("${:.2}/1M", model.input_cost_per_1m));
        } else {
            // Different prices - show compact format
            parts.push(format!("${:.2}⇄${:.2}", model.input_cost_per_1m, model.output_cost_per_1m));
        }
    } else {
        parts.push("Free".to_string());
    }
    
    // Capability indicators
    let mut capabilities = Vec::new();
    if model.supports_tools {
        capabilities.push("🔧"); // Tools/Functions
    }
    if model.supports_streaming {
        capabilities.push("⚡"); // Streaming
    }
    
    if !capabilities.is_empty() {
        parts.push(capabilities.join(""));
    }
    
    if !parts.is_empty() {
        Some(parts.join(" • "))
    } else {
        None
    }
}