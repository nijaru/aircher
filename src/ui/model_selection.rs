use ratatui::{
    layout::Rect,
    Frame,
};
use std::collections::HashMap;

use crate::config::{ConfigManager, ModelConfig};
use crate::providers::ProviderManager;
use crate::auth::{AuthManager, AuthStatus};
use super::typeahead::{TypeaheadOverlay, TypeaheadItem};
use std::sync::Arc;

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
            provider_models.insert(name.clone(), provider_config.models.clone());
        }

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
        };

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
        self.provider_manager = Some(provider_manager);
    }

    fn update_items(&mut self, config: &ConfigManager) {
        // For now, use the old method when no auth manager is available
        // This will be updated to be async when we refactor the callers
        if self.auth_manager.is_none() {
            let provider_items: Vec<TypeaheadItem> = config.providers.keys()
                .map(|name| TypeaheadItem {
                    label: format_provider_name_with_status_legacy(name, config),
                    value: name.clone(),
                    description: Some(format_provider_description_with_auth_legacy(name, config)),
                    available: config.providers.get(name)
                        .map(|p| p.api_key_env.is_empty() || std::env::var(&p.api_key_env).is_ok())
                        .unwrap_or(false),
                })
                .collect();

            self.provider_typeahead.set_items(provider_items);
            self.provider_typeahead.set_current_value(Some(self.current_provider.clone()));
            self.update_provider_description();

            // Update model items for current provider
            self.update_model_items();
            return;
        }

        // Schedule async update - for now we'll use the legacy method as fallback
        // TODO: Make this properly async
        self.update_items_legacy(config);
    }

    fn update_items_legacy(&mut self, config: &ConfigManager) {
        let provider_items: Vec<TypeaheadItem> = config.providers.keys()
            .map(|name| TypeaheadItem {
                label: format_provider_name_with_status_legacy(name, config),
                value: name.clone(),
                description: Some(format_provider_description_with_auth_legacy(name, config)),
                available: config.providers.get(name)
                    .map(|p| p.api_key_env.is_empty() || std::env::var(&p.api_key_env).is_ok())
                    .unwrap_or(false),
            })
            .collect();

        self.provider_typeahead.set_items(provider_items);
        self.provider_typeahead.set_current_value(Some(self.current_provider.clone()));
        self.update_provider_description();

        // Update model items for current provider
        self.update_model_items();
    }

    pub async fn update_items_with_auth(&mut self, config: &ConfigManager) {
        if let Some(auth_manager) = &self.auth_manager {
            let auth_statuses = auth_manager.get_all_provider_statuses(config).await;
            
            let provider_items: Vec<TypeaheadItem> = config.providers.keys()
                .map(|name| {
                    let auth_status = auth_statuses.get(name)
                        .map(|info| &info.status)
                        .unwrap_or(&AuthStatus::NotConfigured);
                    
                    TypeaheadItem {
                        label: format_provider_name_with_auth_status(name, auth_status),
                        value: name.clone(),
                        description: Some(format_provider_description_with_auth_status(name, auth_status, config)),
                        available: matches!(auth_status, AuthStatus::Authenticated) || 
                                 config.providers.get(name)
                                     .map(|p| p.api_key_env.is_empty())
                                     .unwrap_or(false),
                    }
                })
                .collect();

            self.provider_typeahead.set_items(provider_items);
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
        
        if let Some(items) = self.provider_typeahead.items.clone().into_iter()
            .map(|mut item| {
                item.available = provider_list.contains(&item.value);
                Some(item)
            })
            .collect::<Option<Vec<_>>>() {
            self.provider_typeahead.set_items(items);
        }
    }

    /// Update model lists with dynamic data from providers
    pub fn update_dynamic_models(&mut self, providers: &ProviderManager) {
        // For Ollama, get dynamic model list instead of using config
        let ollama_models = providers.get_provider_models("ollama");
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
        // Check if provider is authenticated first
        let is_authenticated = self.provider_typeahead.items.iter()
            .find(|item| item.value == self.current_provider)
            .map(|item| item.available)
            .unwrap_or(false);

        if !is_authenticated {
            // Show message that auth is required
            let no_auth_item = TypeaheadItem {
                label: "Authentication required".to_string(),
                value: "_no_auth".to_string(),
                description: Some("Run /auth to set up API keys".to_string()),
                available: false,
            };
            self.model_typeahead.set_items(vec![no_auth_item]);
            self.model_typeahead.set_current_value(None);
            self.update_model_description();
            return;
        }

        let models = self.provider_models.get(&self.current_provider)
            .cloned()
            .unwrap_or_default();

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

        self.model_typeahead.set_items(model_items);
        self.model_typeahead.set_current_value(Some(self.current_model.clone()));
        self.update_model_description();
    }

    fn update_provider_description(&mut self) {
        self.provider_typeahead.description = format!(
            "Current provider: {}\n\n[ Tab ] or [←/→] Switch to model selection", 
            format_provider_name(&self.current_provider)
        );
    }

    fn update_model_description(&mut self) {
        self.model_typeahead.description = format!(
            "Current model: {}\n\n[ Tab ] or [←/→] Switch to provider selection",
            &self.current_model
        );
    }

    pub fn show(&mut self) {
        // Smart default: check if current provider is authenticated
        let should_start_with_provider = !self.is_current_provider_authenticated();
        
        if should_start_with_provider && self.mode == SelectionMode::Model {
            // Switch to provider selection if current provider not authenticated
            self.mode = SelectionMode::Provider;
        }
        
        // Trigger model fetching for current provider if authenticated
        if self.is_current_provider_authenticated() {
            self.fetch_models_for_provider(&self.current_provider.clone());
        }
        
        match self.mode {
            SelectionMode::Model => self.model_typeahead.show(),
            SelectionMode::Provider => self.provider_typeahead.show(),
        }
    }
    
    fn is_current_provider_authenticated(&self) -> bool {
        self.provider_typeahead.items.iter()
            .find(|item| item.value == self.current_provider)
            .map(|item| item.available)
            .unwrap_or(false)
    }

    fn fetch_models_for_provider(&mut self, provider_name: &str) {
        // Don't fetch if already fetching or no provider manager
        if self.fetching_models || self.provider_manager.is_none() {
            return;
        }

        // Don't fetch if provider not authenticated
        let is_authenticated = self.provider_typeahead.items.iter()
            .find(|item| item.value == provider_name)
            .map(|item| item.available)
            .unwrap_or(false);

        if !is_authenticated {
            return;
        }

        self.fetching_models = true;
        self.models_fetched_for = Some(provider_name.to_string());
        
        // Update model typeahead to show loading state
        let loading_item = TypeaheadItem {
            label: "Loading models...".to_string(),
            value: "_loading".to_string(),
            description: Some("Fetching available models from provider".to_string()),
            available: false,
        };
        self.model_typeahead.set_items(vec![loading_item]);
        self.update_model_description();

        // Note: In a real implementation, we'd spawn a background task here
        // For now, we'll add a placeholder that can be filled in when we have async context
        // TODO: Implement actual async model fetching
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
                    None // Don't return selection yet, let user pick model
                } else {
                    None
                }
            }
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
        "ollama" => "Ollama (Local)".to_string(),
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
    let status_icon = get_auth_status_icon(auth_status);
    format!("{} {}", status_icon, base_name)
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
            "⚡" // Local provider (no auth needed)
        } else if std::env::var(&provider_config.api_key_env).is_ok() {
            "✓" // Authenticated
        } else {
            "❌" // Needs setup
        }
    } else {
        "○" // Not configured
    }
}

fn get_provider_auth_description_legacy(provider: &str, config: &ConfigManager) -> String {
    if let Some(provider_config) = config.get_provider(provider) {
        if provider_config.api_key_env.is_empty() {
            "local (no auth needed)".to_string()
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
        AuthStatus::NotConfigured => "○",
        AuthStatus::Invalid => "❌",
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
        AuthStatus::NetworkError => "network error",
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