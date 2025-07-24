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
        };

        overlay.update_items(config);
        overlay
    }

    pub fn with_providers(config: &ConfigManager, providers: &ProviderManager) -> Self {
        let mut overlay = Self::new(config);
        overlay.has_providers = true;
        overlay.update_provider_availability(providers);
        overlay
    }

    pub fn with_auth_manager(config: &ConfigManager, auth_manager: Arc<AuthManager>) -> Self {
        let mut overlay = Self::new(config);
        overlay.auth_manager = Some(auth_manager);
        overlay.update_items(config);
        overlay
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

    fn update_provider_availability(&mut self, providers: &ProviderManager) {
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

    fn update_model_items(&mut self) {
        let models = self.provider_models.get(&self.current_provider)
            .cloned()
            .unwrap_or_default();

        let mut model_items: Vec<TypeaheadItem> = Vec::new();
        
        // Add "Default" option for Anthropic provider (similar to Claude Code)
        if self.current_provider == "anthropic" {
            model_items.push(TypeaheadItem {
                label: "Default (Opus 4 for up to 50% of usage limits, then use Sonnet 4)".to_string(),
                value: "default".to_string(),
                description: Some("Smart model selection based on usage limits".to_string()),
                available: true,
            });
        }
        
        // Add regular models
        model_items.extend(models.into_iter()
            .map(|model| TypeaheadItem {
                label: model.name.clone(),
                value: model.name.clone(),
                description: format_model_description(&model),
                available: true, // Models are available if provider is configured
            }));

        self.model_typeahead.set_items(model_items);
        self.model_typeahead.set_current_value(Some(self.current_model.clone()));
        self.update_model_description();
    }

    fn update_provider_description(&mut self) {
        self.provider_typeahead.description = format!(
            "Current provider: {}\n{}",
            format_provider_name(&self.current_provider),
            "Press Tab to switch to model selection"
        );
    }

    fn update_model_description(&mut self) {
        self.model_typeahead.description = format!(
            "Current model: {}\n{}",
            &self.current_model,
            "Press Tab to switch to provider selection"
        );
    }

    pub fn show(&mut self) {
        match self.mode {
            SelectionMode::Model => self.model_typeahead.show(),
            SelectionMode::Provider => self.provider_typeahead.show(),
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

    pub fn move_selection_up(&mut self) {
        match self.mode {
            SelectionMode::Model => self.model_typeahead.move_selection_up(),
            SelectionMode::Provider => self.provider_typeahead.move_selection_up(),
        }
    }

    pub fn move_selection_down(&mut self) {
        match self.mode {
            SelectionMode::Model => self.model_typeahead.move_selection_down(),
            SelectionMode::Provider => self.provider_typeahead.move_selection_down(),
        }
    }

    pub fn get_selected(&mut self) -> Option<(String, String)> {
        match self.mode {
            SelectionMode::Model => {
                if let Some(item) = self.model_typeahead.get_selected() {
                    Some((self.current_provider.clone(), item.value.clone()))
                } else {
                    None
                }
            }
            SelectionMode::Provider => {
                if let Some(item) = self.provider_typeahead.get_selected() {
                    // When provider is selected, update model list and switch to model selection
                    self.current_provider = item.value.clone();
                    self.update_model_items();
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
        "claude" => "Anthropic Claude".to_string(),
        "openai" => "OpenAI".to_string(),
        "gemini" => "Google Gemini".to_string(),
        "ollama" => "Ollama (Local)".to_string(),
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
        "claude" => "Best for complex reasoning and coding".to_string(),
        "openai" => "GPT models with broad capabilities".to_string(),
        "gemini" => "Google's multimodal AI models".to_string(),
        "ollama" => "Run models locally on your machine".to_string(),
        "openrouter" => "Access multiple providers through one API".to_string(),
        _ => String::new(),
    }
}

fn format_model_description(model: &ModelConfig) -> Option<String> {
    let mut parts = Vec::new();
    
    if model.context_window > 0 {
        parts.push(format!("{}k context", model.context_window / 1000));
    }
    
    // Add pricing info if available (non-zero)
    if model.input_cost_per_1m > 0.0 || model.output_cost_per_1m > 0.0 {
        parts.push(format!("${:.4}/${:.4} per 1M", model.input_cost_per_1m, model.output_cost_per_1m));
    }
    
    if !parts.is_empty() {
        Some(parts.join(" • "))
    } else {
        None
    }
}