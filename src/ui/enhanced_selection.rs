use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use tracing::info;

use crate::config::ConfigManager;
use crate::providers::ProviderManager;

/// Enhanced multi-level model selection modal with fuzzy filtering
pub struct EnhancedSelectionModal {
    visible: bool,
    current_level: SelectionLevel,
    providers: Vec<ProviderInfo>,
    models: Vec<ModelInfo>,
    hosts: Vec<HostInfo>,
    selected_provider_idx: usize,
    selected_model_idx: usize,
    selected_host_idx: usize,
    filter_text: String,
    filtered_indices: Vec<usize>,
    auth_input: String,
    auth_input_visible: bool,
    needs_auth: bool,
    provider_manager: Option<ProviderManager>,
    config_manager: Option<ConfigManager>,
}

#[derive(Debug, Clone, PartialEq)]
enum SelectionLevel {
    Provider,
    Model,
    Host,
    Auth,
    Confirmation,
}

#[derive(Debug, Clone)]
struct ProviderInfo {
    name: String,
    display_name: String,
    icon: String,
    is_authenticated: bool,
    is_available: bool,
    description: String,
}

#[derive(Debug, Clone)]
struct ModelInfo {
    name: String,
    display_name: String,
    provider: String,
    context_window: u32,
    input_cost_per_1m: f64,
    output_cost_per_1m: f64,
    description: String,
}

#[derive(Debug, Clone)]
struct HostInfo {
    name: String,
    display_name: String,
    provider: String,
    is_default: bool,
    description: String,
}

#[derive(Debug, Clone)]
pub struct SelectionResult {
    pub provider: String,
    pub model: String,
    pub host: Option<String>,
}

impl EnhancedSelectionModal {
    pub fn new() -> Self {
        Self {
            visible: false,
            current_level: SelectionLevel::Provider,
            providers: Vec::new(),
            models: Vec::new(),
            hosts: Vec::new(),
            selected_provider_idx: 0,
            selected_model_idx: 0,
            selected_host_idx: 0,
            filter_text: String::new(),
            filtered_indices: Vec::new(),
            auth_input: String::new(),
            auth_input_visible: false,
            needs_auth: false,
            provider_manager: None,
            config_manager: None,
        }
    }
    
    pub async fn initialize(&mut self, providers: &ProviderManager, config: &ConfigManager) -> Result<()> {
        self.providers = self.load_providers(providers, config).await?;
        self.filtered_indices = (0..self.providers.len()).collect();
        Ok(())
    }
    
    async fn load_providers(&self, _providers: &ProviderManager, config: &ConfigManager) -> Result<Vec<ProviderInfo>> {
        let mut provider_list = Vec::new();
        
        // Claude
        provider_list.push(ProviderInfo {
            name: "claude".to_string(),
            display_name: "Claude (Anthropic)".to_string(),
            icon: "🤖".to_string(),
            is_authenticated: config.has_api_key("claude"),
            is_available: true,
            description: "High-quality AI with large context windows".to_string(),
        });
        
        // OpenAI
        provider_list.push(ProviderInfo {
            name: "openai".to_string(),
            display_name: "OpenAI".to_string(),
            icon: "🧠".to_string(),
            is_authenticated: config.has_api_key("openai"),
            is_available: true,
            description: "GPT models with strong reasoning capabilities".to_string(),
        });
        
        // Gemini
        provider_list.push(ProviderInfo {
            name: "gemini".to_string(),
            display_name: "Gemini (Google)".to_string(),
            icon: "⭐".to_string(),
            is_authenticated: config.has_api_key("gemini"),
            is_available: true,
            description: "Google's latest AI with long context support".to_string(),
        });
        
        // Ollama
        let ollama_available = self.check_ollama_availability().await;
        provider_list.push(ProviderInfo {
            name: "ollama".to_string(),
            display_name: "Ollama (Local)".to_string(),
            icon: "🏠".to_string(),
            is_authenticated: true,  // No auth needed
            is_available: ollama_available,
            description: "Local models, free and private".to_string(),
        });
        
        // OpenRouter
        provider_list.push(ProviderInfo {
            name: "openrouter".to_string(),
            display_name: "OpenRouter".to_string(),
            icon: "🌐".to_string(),
            is_authenticated: config.has_api_key("openrouter"),
            is_available: true,
            description: "Access to many models via unified API".to_string(),
        });
        
        Ok(provider_list)
    }
    
    async fn check_ollama_availability(&self) -> bool {
        // In a real implementation, this would check if Ollama is running
        // For now, return true as a placeholder
        true
    }
    
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.reset_state();
        }
    }
    
    pub fn show(&mut self) {
        self.visible = true;
        self.reset_state();
    }
    
    pub fn hide(&mut self) {
        self.visible = false;
    }
    
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    fn reset_state(&mut self) {
        self.current_level = SelectionLevel::Provider;
        self.selected_provider_idx = 0;
        self.selected_model_idx = 0;
        self.selected_host_idx = 0;
        self.filter_text.clear();
        self.filtered_indices = (0..self.providers.len()).collect();
        self.auth_input.clear();
        self.auth_input_visible = false;
        self.needs_auth = false;
    }
    
    /// Handle navigation input
    pub async fn handle_input(&mut self, key: char) -> Result<Option<SelectionResult>> {
        match self.current_level {
            SelectionLevel::Auth => {
                self.handle_auth_input(key).await
            },
            _ => {
                self.handle_navigation_input(key).await
            }
        }
    }
    
    async fn handle_navigation_input(&mut self, key: char) -> Result<Option<SelectionResult>> {
        match key {
            // Navigation
            'j' | 'J' => {
                self.move_down();
                Ok(None)
            },
            'k' | 'K' => {
                self.move_up();
                Ok(None)
            },
            'h' | 'H' => {
                self.move_back().await?;
                Ok(None)
            },
            'l' | 'L' => self.move_forward().await,
            '\n' | '\r' => self.move_forward().await,  // Enter
            '\x1b' => {
                self.move_back().await?;
                Ok(None)
            },  // Esc
            
            // Filtering (only for model selection on large lists)
            c if c.is_alphanumeric() || c == ' ' || c == '-' => {
                if self.should_enable_filtering() {
                    self.filter_text.push(c);
                    self.update_filtered_indices();
                }
                Ok(None)
            },
            
            // Backspace in filter
            '\x7f' | '\x08' => {  // Backspace/Delete
                if self.should_enable_filtering() && !self.filter_text.is_empty() {
                    self.filter_text.pop();
                    self.update_filtered_indices();
                }
                Ok(None)
            },
            
            _ => Ok(None),
        }
    }
    
    async fn handle_auth_input(&mut self, key: char) -> Result<Option<SelectionResult>> {
        match key {
            '\n' | '\r' => {  // Enter - save auth
                self.save_auth_key().await?;
                self.auth_input_visible = false;
                self.current_level = SelectionLevel::Model;
                self.load_models_for_provider().await?;
                Ok(None)
            },
            '\x1b' => {  // Esc - cancel auth
                self.auth_input.clear();
                self.auth_input_visible = false;
                self.current_level = SelectionLevel::Provider;
                Ok(None)
            },
            '\x7f' | '\x08' => {  // Backspace
                self.auth_input.pop();
                Ok(None)
            },
            c if c.is_ascii() => {
                self.auth_input.push(c);
                Ok(None)
            },
            _ => Ok(None),
        }
    }
    
    fn should_enable_filtering(&self) -> bool {
        matches!(self.current_level, SelectionLevel::Model) && 
        self.get_current_provider().map(|p| p.name == "openrouter").unwrap_or(false)
    }
    
    fn move_up(&mut self) {
        let max_idx = match self.current_level {
            SelectionLevel::Provider => self.filtered_indices.len(),
            SelectionLevel::Model => self.filtered_indices.len(),
            SelectionLevel::Host => self.hosts.len(),
            _ => return,
        };
        
        if max_idx == 0 { return; }
        
        match self.current_level {
            SelectionLevel::Provider => {
                self.selected_provider_idx = if self.selected_provider_idx > 0 {
                    self.selected_provider_idx - 1
                } else {
                    max_idx - 1
                };
            },
            SelectionLevel::Model => {
                self.selected_model_idx = if self.selected_model_idx > 0 {
                    self.selected_model_idx - 1
                } else {
                    max_idx - 1
                };
            },
            SelectionLevel::Host => {
                self.selected_host_idx = if self.selected_host_idx > 0 {
                    self.selected_host_idx - 1
                } else {
                    max_idx - 1
                };
            },
            _ => {},
        }
    }
    
    fn move_down(&mut self) {
        let max_idx = match self.current_level {
            SelectionLevel::Provider => self.filtered_indices.len(),
            SelectionLevel::Model => self.filtered_indices.len(),
            SelectionLevel::Host => self.hosts.len(),
            _ => return,
        };
        
        if max_idx == 0 { return; }
        
        match self.current_level {
            SelectionLevel::Provider => {
                self.selected_provider_idx = (self.selected_provider_idx + 1) % max_idx;
            },
            SelectionLevel::Model => {
                self.selected_model_idx = (self.selected_model_idx + 1) % max_idx;
            },
            SelectionLevel::Host => {
                self.selected_host_idx = (self.selected_host_idx + 1) % max_idx;
            },
            _ => {},
        }
    }
    
    async fn move_forward(&mut self) -> Result<Option<SelectionResult>> {
        match self.current_level {
            SelectionLevel::Provider => {
                let provider = self.get_current_provider().cloned();
                if let Some(provider_info) = provider {
                    if !provider_info.is_available {
                        return Ok(None);  // Can't select unavailable provider
                    }
                    
                    if !provider_info.is_authenticated && provider_info.name != "ollama" {
                        self.current_level = SelectionLevel::Auth;
                        self.auth_input_visible = true;
                        self.needs_auth = true;
                    } else {
                        self.current_level = SelectionLevel::Model;
                        self.load_models_for_provider().await?;
                    }
                }
                Ok(None)
            },
            SelectionLevel::Model => {
                let provider = self.get_current_provider().cloned();
                if let Some(provider_info) = provider {
                    if provider_info.name == "openrouter" {
                        // Auto-advance to host selection for OpenRouter
                        self.current_level = SelectionLevel::Host;
                        self.load_hosts_for_model().await?;
                    } else {
                        // Direct completion for other providers
                        return self.create_selection_result();
                    }
                }
                Ok(None)
            },
            SelectionLevel::Host => {
                // Final selection
                self.create_selection_result()
            },
            _ => Ok(None),
        }
    }
    
    async fn move_back(&mut self) -> Result<()> {
        match self.current_level {
            SelectionLevel::Auth => {
                self.current_level = SelectionLevel::Provider;
                self.auth_input.clear();
                self.auth_input_visible = false;
            },
            SelectionLevel::Model => {
                self.current_level = SelectionLevel::Provider;
                self.filter_text.clear();
                self.update_filtered_indices();
            },
            SelectionLevel::Host => {
                self.current_level = SelectionLevel::Model;
            },
            SelectionLevel::Provider => {
                self.hide();
            },
            _ => {},
        }
        Ok(())
    }
    
    fn get_current_provider(&self) -> Option<&ProviderInfo> {
        self.filtered_indices.get(self.selected_provider_idx)
            .and_then(|&idx| self.providers.get(idx))
    }
    
    fn get_current_model(&self) -> Option<&ModelInfo> {
        self.filtered_indices.get(self.selected_model_idx)
            .and_then(|&idx| self.models.get(idx))
    }
    
    fn get_current_host(&self) -> Option<&HostInfo> {
        self.hosts.get(self.selected_host_idx)
    }
    
    async fn load_models_for_provider(&mut self) -> Result<()> {
        let provider = self.get_current_provider().cloned();
        if let Some(provider_info) = provider {
            self.models = self.fetch_models_for_provider(&provider_info.name).await?;
            self.selected_model_idx = 0;
            self.update_filtered_indices();
        }
        Ok(())
    }
    
    async fn fetch_models_for_provider(&self, provider_name: &str) -> Result<Vec<ModelInfo>> {
        // Mock implementation - in real version would fetch from provider
        match provider_name {
            "claude" => Ok(vec![
                ModelInfo {
                    name: "claude-3-5-sonnet-20241022".to_string(),
                    display_name: "Claude 3.5 Sonnet".to_string(),
                    provider: "claude".to_string(),
                    context_window: 200000,
                    input_cost_per_1m: 3.0,
                    output_cost_per_1m: 15.0,
                    description: "Most capable Claude model".to_string(),
                },
                ModelInfo {
                    name: "claude-3-haiku-20240307".to_string(),
                    display_name: "Claude 3 Haiku".to_string(),
                    provider: "claude".to_string(),
                    context_window: 200000,
                    input_cost_per_1m: 0.25,
                    output_cost_per_1m: 1.25,
                    description: "Fast and cost-effective".to_string(),
                },
            ]),
            "openai" => Ok(vec![
                ModelInfo {
                    name: "gpt-4o".to_string(),
                    display_name: "GPT-4o".to_string(),
                    provider: "openai".to_string(),
                    context_window: 128000,
                    input_cost_per_1m: 2.5,
                    output_cost_per_1m: 10.0,
                    description: "Most capable OpenAI model".to_string(),
                },
                ModelInfo {
                    name: "gpt-4o-mini".to_string(),
                    display_name: "GPT-4o Mini".to_string(),
                    provider: "openai".to_string(),
                    context_window: 128000,
                    input_cost_per_1m: 0.15,
                    output_cost_per_1m: 0.6,
                    description: "Cost-effective GPT-4 class model".to_string(),
                },
            ]),
            "openrouter" => {
                // Large list for fuzzy filtering demonstration
                Ok(vec![
                    ModelInfo {
                        name: "anthropic/claude-3-5-sonnet".to_string(),
                        display_name: "Claude 3.5 Sonnet".to_string(),
                        provider: "openrouter".to_string(),
                        context_window: 200000,
                        input_cost_per_1m: 3.0,
                        output_cost_per_1m: 15.0,
                        description: "Via Anthropic".to_string(),
                    },
                    ModelInfo {
                        name: "openai/gpt-4o".to_string(),
                        display_name: "GPT-4o".to_string(),
                        provider: "openrouter".to_string(),
                        context_window: 128000,
                        input_cost_per_1m: 2.5,
                        output_cost_per_1m: 10.0,
                        description: "Via OpenAI".to_string(),
                    },
                    ModelInfo {
                        name: "meta-llama/llama-3.3-70b-instruct".to_string(),
                        display_name: "Llama 3.3 70B".to_string(),
                        provider: "openrouter".to_string(),
                        context_window: 131072,
                        input_cost_per_1m: 0.59,
                        output_cost_per_1m: 0.79,
                        description: "Meta's latest model".to_string(),
                    },
                    ModelInfo {
                        name: "google/gemini-2.0-flash-exp".to_string(),
                        display_name: "Gemini 2.0 Flash".to_string(),
                        provider: "openrouter".to_string(),
                        context_window: 1000000,
                        input_cost_per_1m: 0.075,
                        output_cost_per_1m: 0.3,
                        description: "Via Google".to_string(),
                    },
                ])
            },
            _ => Ok(vec![]),
        }
    }
    
    async fn load_hosts_for_model(&mut self) -> Result<()> {
        let model = self.get_current_model().cloned();
        if let Some(model_info) = model {
            self.hosts = self.fetch_hosts_for_model(&model_info.name).await?;
            
            // Find default host and highlight it
            if let Some(default_idx) = self.hosts.iter().position(|h| h.is_default) {
                self.selected_host_idx = default_idx;
            } else {
                self.selected_host_idx = 0;
            }
        }
        Ok(())
    }
    
    async fn fetch_hosts_for_model(&self, model_name: &str) -> Result<Vec<HostInfo>> {
        // Mock implementation - in real version would fetch from OpenRouter API
        if model_name.starts_with("anthropic/") {
            Ok(vec![
                HostInfo {
                    name: "anthropic".to_string(),
                    display_name: "Default (Anthropic)".to_string(),
                    provider: "anthropic".to_string(),
                    is_default: true,
                    description: "Official Anthropic API".to_string(),
                },
            ])
        } else if model_name.starts_with("openai/") {
            Ok(vec![
                HostInfo {
                    name: "openai".to_string(),
                    display_name: "Default (OpenAI)".to_string(),
                    provider: "openai".to_string(),
                    is_default: true,
                    description: "Official OpenAI API".to_string(),
                },
            ])
        } else if model_name.starts_with("meta-llama/") {
            Ok(vec![
                HostInfo {
                    name: "together".to_string(),
                    display_name: "Default (Together AI)".to_string(),
                    provider: "together".to_string(),
                    is_default: true,
                    description: "Together AI hosting".to_string(),
                },
                HostInfo {
                    name: "fireworks".to_string(),
                    display_name: "Fireworks AI".to_string(),
                    provider: "fireworks".to_string(),
                    is_default: false,
                    description: "Fast inference hosting".to_string(),
                },
            ])
        } else {
            Ok(vec![
                HostInfo {
                    name: "default".to_string(),
                    display_name: "Default".to_string(),
                    provider: "default".to_string(),
                    is_default: true,
                    description: "Default hosting".to_string(),
                },
            ])
        }
    }
    
    fn update_filtered_indices(&mut self) {
        if self.filter_text.is_empty() {
            match self.current_level {
                SelectionLevel::Provider => {
                    self.filtered_indices = (0..self.providers.len()).collect();
                },
                SelectionLevel::Model => {
                    self.filtered_indices = (0..self.models.len()).collect();
                },
                _ => {},
            }
        } else {
            let filter_lower = self.filter_text.to_lowercase();
            match self.current_level {
                SelectionLevel::Provider => {
                    self.filtered_indices = self.providers
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| {
                            p.name.to_lowercase().contains(&filter_lower) ||
                            p.display_name.to_lowercase().contains(&filter_lower)
                        })
                        .map(|(i, _)| i)
                        .collect();
                },
                SelectionLevel::Model => {
                    self.filtered_indices = self.models
                        .iter()
                        .enumerate()
                        .filter(|(_, m)| {
                            m.name.to_lowercase().contains(&filter_lower) ||
                            m.display_name.to_lowercase().contains(&filter_lower) ||
                            m.description.to_lowercase().contains(&filter_lower)
                        })
                        .map(|(i, _)| i)
                        .collect();
                },
                _ => {},
            }
        }
        
        // Reset selection to first filtered item
        match self.current_level {
            SelectionLevel::Provider => self.selected_provider_idx = 0,
            SelectionLevel::Model => self.selected_model_idx = 0,
            _ => {},
        }
    }
    
    async fn save_auth_key(&mut self) -> Result<()> {
        // In real implementation, would save to config
        info!("Saved auth key for provider");
        Ok(())
    }
    
    fn create_selection_result(&self) -> Result<Option<SelectionResult>> {
        let provider = self.get_current_provider()
            .ok_or_else(|| anyhow::anyhow!("No provider selected"))?;
        let model = self.get_current_model()
            .ok_or_else(|| anyhow::anyhow!("No model selected"))?;
        
        let host = if provider.name == "openrouter" {
            self.get_current_host().map(|h| h.name.clone())
        } else {
            None
        };
        
        Ok(Some(SelectionResult {
            provider: provider.name.clone(),
            model: model.name.clone(),
            host,
        }))
    }
    
    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }
        
        // Calculate modal area
        let modal_area = centered_rect(85, 80, area);
        
        // Clear background
        f.render_widget(Clear, modal_area);
        
        // Render modal frame
        let title = match self.current_level {
            SelectionLevel::Provider => "Select Provider",
            SelectionLevel::Model => "Select Model",
            SelectionLevel::Host => "Select Host",
            SelectionLevel::Auth => "Enter API Key",
            SelectionLevel::Confirmation => "Confirm Selection",
        };
        
        let modal_block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::default().bg(Color::Black));
        f.render_widget(modal_block, modal_area);
        
        // Inner area
        let inner_area = Rect {
            x: modal_area.x + 1,
            y: modal_area.y + 1,
            width: modal_area.width.saturating_sub(2),
            height: modal_area.height.saturating_sub(2),
        };
        
        // Split areas
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Content
                Constraint::Length(4), // Instructions
            ])
            .split(inner_area);
        
        // Render content based on current level
        match self.current_level {
            SelectionLevel::Provider => self.render_providers(f, chunks[0]),
            SelectionLevel::Model => self.render_models(f, chunks[0]),
            SelectionLevel::Host => self.render_hosts(f, chunks[0]),
            SelectionLevel::Auth => self.render_auth(f, chunks[0]),
            SelectionLevel::Confirmation => self.render_confirmation(f, chunks[0]),
        }
        
        // Render instructions
        self.render_instructions(f, chunks[1]);
    }
    
    fn render_providers(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.filtered_indices
            .iter()
            .enumerate()
            .filter_map(|(display_idx, &provider_idx)| {
                self.providers.get(provider_idx).map(|provider| {
                    let style = if display_idx == self.selected_provider_idx {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else {
                        Style::default()
                    };
                    
                    let auth_status = if provider.is_authenticated {
                        "✅"
                    } else {
                        "❌"
                    };
                    
                    let availability = if provider.is_available {
                        ""
                    } else {
                        " (unavailable)"
                    };
                    
                    let text = format!(
                        "  {} {} {} {}{}",
                        provider.icon,
                        provider.display_name,
                        auth_status,
                        provider.description,
                        availability
                    );
                    
                    ListItem::new(text).style(style)
                })
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Available Providers"));
        f.render_widget(list, area);
    }
    
    fn render_models(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Filter input
                Constraint::Min(0),    // Model list
            ])
            .split(area);
        
        // Filter input (for OpenRouter)
        if self.should_enable_filtering() {
            let filter_text = if self.filter_text.is_empty() {
                "Type to filter models..."
            } else {
                &self.filter_text
            };
            
            let filter_widget = Paragraph::new(filter_text)
                .style(Style::default().fg(if self.filter_text.is_empty() { Color::DarkGray } else { Color::White }))
                .block(Block::default().borders(Borders::ALL).title("Filter"));
            f.render_widget(filter_widget, chunks[0]);
        }
        
        let content_area = if self.should_enable_filtering() { chunks[1] } else { area };
        
        let items: Vec<ListItem> = self.filtered_indices
            .iter()
            .enumerate()
            .filter_map(|(display_idx, &model_idx)| {
                self.models.get(model_idx).map(|model| {
                    let style = if display_idx == self.selected_model_idx {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else {
                        Style::default()
                    };
                    
                    let text = format!(
                        "  {} ({}k ctx, ${:.2}/${:.2} per 1M)",
                        model.display_name,
                        model.context_window / 1000,
                        model.input_cost_per_1m,
                        model.output_cost_per_1m
                    );
                    
                    ListItem::new(text).style(style)
                })
            })
            .collect();
        
        let current_provider = self.get_current_provider()
            .map(|p| p.display_name.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(format!("Models for {}", current_provider)));
        f.render_widget(list, content_area);
    }
    
    fn render_hosts(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.hosts
            .iter()
            .enumerate()
            .map(|(idx, host)| {
                let style = if idx == self.selected_host_idx {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                
                let default_marker = if host.is_default { " ⭐" } else { "" };
                let text = format!("  {}{} - {}", host.display_name, default_marker, host.description);
                
                ListItem::new(text).style(style)
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Available Hosts"));
        f.render_widget(list, area);
    }
    
    fn render_auth(&self, f: &mut Frame, area: Rect) {
        let masked_input = "*".repeat(self.auth_input.len());
        
        let input_widget = Paragraph::new(masked_input)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("API Key"));
        f.render_widget(input_widget, area);
    }
    
    fn render_confirmation(&self, f: &mut Frame, area: Rect) {
        let provider = self.get_current_provider();
        let model = self.get_current_model();
        let host = self.get_current_host();
        
        let text = if let (Some(p), Some(m)) = (provider, model) {
            if let Some(h) = host {
                format!("Provider: {}\nModel: {}\nHost: {}\n\nPress Enter to confirm, Esc to cancel", 
                       p.display_name, m.display_name, h.display_name)
            } else {
                format!("Provider: {}\nModel: {}\n\nPress Enter to confirm, Esc to cancel", 
                       p.display_name, m.display_name)
            }
        } else {
            "Invalid selection".to_string()
        };
        
        let confirmation_widget = Paragraph::new(text)
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title("Confirm Selection"));
        f.render_widget(confirmation_widget, area);
    }
    
    fn render_instructions(&self, f: &mut Frame, area: Rect) {
        let instructions = match self.current_level {
            SelectionLevel::Provider => "↑↓ Navigate, →/Enter Advance, Esc Cancel",
            SelectionLevel::Model => {
                if self.should_enable_filtering() {
                    "Type to filter, ↑↓ Navigate, →/Enter Advance, ←/Esc Back"
                } else {
                    "↑↓ Navigate, →/Enter Advance, ←/Esc Back"
                }
            },
            SelectionLevel::Host => "↑↓ Navigate, →/Enter Confirm, ←/Esc Back",
            SelectionLevel::Auth => "Type API key, Enter Save, Esc Cancel",
            SelectionLevel::Confirmation => "Enter Confirm, Esc Cancel",
        };
        
        let instructions_widget = Paragraph::new(instructions)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(instructions_widget, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

impl Default for EnhancedSelectionModal {
    fn default() -> Self {
        Self::new()
    }
}