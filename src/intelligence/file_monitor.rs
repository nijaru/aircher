use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, sleep};
use tracing::{debug, info, warn};

use crate::project::ProjectManager;
use super::tools::IntelligenceTools;
use super::tui_tools::TuiIntelligenceTools;

/// File change event types
#[derive(Debug, Clone)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf, to: PathBuf },
}

/// File change event
#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub change_type: FileChangeType,
    pub timestamp: SystemTime,
}

/// File metadata for tracking changes
#[derive(Debug, Clone)]
struct FileMetadata {
    path: PathBuf,
    modified: SystemTime,
    size: u64,
    exists: bool,
}

/// Background file monitor for intelligent analysis
pub struct FileMonitor {
    project_manager: ProjectManager,
    intelligence_tools: Arc<RwLock<TuiIntelligenceTools>>,
    file_cache: Arc<RwLock<HashMap<PathBuf, FileMetadata>>>,
    change_sender: mpsc::UnboundedSender<FileChangeEvent>,
    change_receiver: Option<mpsc::UnboundedReceiver<FileChangeEvent>>,
    debounce_duration: Duration,
    scan_interval: Duration,
    is_running: Arc<RwLock<bool>>,
}

impl FileMonitor {
    /// Create a new file monitor
    pub fn new(
        project_manager: ProjectManager,
        intelligence_tools: TuiIntelligenceTools,
    ) -> Result<Self> {
        let (change_sender, change_receiver) = mpsc::unbounded_channel();
        
        Ok(Self {
            project_manager,
            intelligence_tools: Arc::new(RwLock::new(intelligence_tools)),
            file_cache: Arc::new(RwLock::new(HashMap::new())),
            change_sender,
            change_receiver: Some(change_receiver),
            debounce_duration: Duration::from_secs(2), // 2 second debounce
            scan_interval: Duration::from_secs(5), // Scan every 5 seconds
            is_running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Start the file monitoring system
    pub async fn start(&mut self) -> Result<()> {
        if *self.is_running.read().await {
            warn!("File monitor is already running");
            return Ok(());
        }
        
        *self.is_running.write().await = true;
        info!("Starting file monitor");
        
        // Initialize file cache
        self.initialize_file_cache().await?;
        
        // Start the file scanning task
        let scanner_handle = self.start_file_scanner().await;
        
        // Start the change processing task
        let processor_handle = self.start_change_processor().await;
        
        // Wait for tasks to complete (they run indefinitely)
        tokio::select! {
            _ = scanner_handle => {
                warn!("File scanner task completed unexpectedly");
            }
            _ = processor_handle => {
                warn!("Change processor task completed unexpectedly");
            }
        }
        
        Ok(())
    }
    
    /// Stop the file monitoring system
    pub async fn stop(&self) {
        *self.is_running.write().await = false;
        info!("Stopping file monitor");
    }
    
    /// Initialize the file cache with current project files
    async fn initialize_file_cache(&self) -> Result<()> {
        debug!("Initializing file cache");
        
        let project_root = self.project_manager.get_project_root()
            .unwrap_or(self.project_manager.get_current_dir());
        
        let files = self.scan_project_files(project_root).await?;
        
        let mut cache = self.file_cache.write().await;
        for file_path in files {
            if let Ok(metadata) = self.get_file_metadata(&file_path).await {
                cache.insert(file_path.clone(), metadata);
            }
        }
        
        info!("Initialized file cache with {} files", cache.len());
        Ok(())
    }
    
    /// Scan project files recursively
    async fn scan_project_files(&self, project_root: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.scan_directory_recursive(project_root, &mut files, 0, 5)?;
        Ok(files)
    }
    
    /// Recursively scan directory for files
    fn scan_directory_recursive(
        &self,
        dir: &Path,
        files: &mut Vec<PathBuf>,
        current_depth: usize,
        max_depth: usize,
    ) -> Result<()> {
        if current_depth > max_depth {
            return Ok(());
        }
        
        let entries = std::fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            // Skip hidden files and directories
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') || self.should_ignore_file(&path) {
                    continue;
                }
            }
            
            if path.is_dir() {
                self.scan_directory_recursive(&path, files, current_depth + 1, max_depth)?;
            } else if path.is_file() {
                files.push(path);
            }
        }
        
        Ok(())
    }
    
    /// Check if file should be ignored
    fn should_ignore_file(&self, path: &Path) -> bool {
        let ignore_patterns = [
            "target", "node_modules", ".git", ".DS_Store",
            ".exe", ".dll", ".so", ".dylib", ".png", ".jpg", ".jpeg", ".gif",
            ".pdf", ".zip", ".tar", ".gz", ".bz2", ".xz", ".lock"
        ];
        
        if let Some(path_str) = path.to_str() {
            ignore_patterns.iter().any(|pattern| path_str.contains(pattern))
        } else {
            false
        }
    }
    
    /// Get file metadata
    async fn get_file_metadata(&self, path: &Path) -> Result<FileMetadata> {
        let metadata = tokio::fs::metadata(path).await?;
        
        Ok(FileMetadata {
            path: path.to_path_buf(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            size: metadata.len(),
            exists: true,
        })
    }
    
    /// Start the file scanner task
    async fn start_file_scanner(&self) -> tokio::task::JoinHandle<()> {
        let project_manager = self.project_manager.clone();
        let file_cache = self.file_cache.clone();
        let change_sender = self.change_sender.clone();
        let scan_interval = self.scan_interval;
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(scan_interval);
            
            while *is_running.read().await {
                interval.tick().await;
                
                if let Err(e) = Self::scan_for_changes(
                    &project_manager,
                    &file_cache,
                    &change_sender,
                ).await {
                    warn!("Error scanning for file changes: {}", e);
                }
            }
        })
    }
    
    /// Start the change processor task
    async fn start_change_processor(&mut self) -> tokio::task::JoinHandle<()> {
        let mut change_receiver = self.change_receiver.take().unwrap();
        let intelligence_tools = self.intelligence_tools.clone();
        let debounce_duration = self.debounce_duration;
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut pending_changes: HashMap<PathBuf, FileChangeEvent> = HashMap::new();
            let mut last_processed = SystemTime::now();
            
            while *is_running.read().await {
                tokio::select! {
                    // Process incoming changes
                    Some(change_event) = change_receiver.recv() => {
                        debug!("Received file change: {:?}", change_event);
                        pending_changes.insert(change_event.path.clone(), change_event);
                    }
                    
                    // Process pending changes after debounce period
                    _ = sleep(debounce_duration) => {
                        if !pending_changes.is_empty() && 
                           last_processed.elapsed().unwrap_or(Duration::ZERO) >= debounce_duration {
                            
                            if let Err(e) = Self::process_pending_changes(
                                &pending_changes,
                                &intelligence_tools,
                            ).await {
                                warn!("Error processing file changes: {}", e);
                            }
                            
                            pending_changes.clear();
                            last_processed = SystemTime::now();
                        }
                    }
                }
            }
        })
    }
    
    /// Scan for file changes
    async fn scan_for_changes(
        project_manager: &ProjectManager,
        file_cache: &Arc<RwLock<HashMap<PathBuf, FileMetadata>>>,
        change_sender: &mpsc::UnboundedSender<FileChangeEvent>,
    ) -> Result<()> {
        let project_root = project_manager.get_project_root()
            .unwrap_or(project_manager.get_current_dir());
        
        // Create a temporary monitor instance to access scan methods
        let temp_monitor = FileMonitor {
            project_manager: project_manager.clone(),
            intelligence_tools: Arc::new(RwLock::new(TuiIntelligenceTools::new()?)),
            file_cache: file_cache.clone(),
            change_sender: change_sender.clone(),
            change_receiver: None,
            debounce_duration: Duration::from_secs(2),
            scan_interval: Duration::from_secs(5),
            is_running: Arc::new(RwLock::new(true)),
        };
        
        let current_files = temp_monitor.scan_project_files(project_root).await?;
        let mut cache = file_cache.write().await;
        
        // Check for new or modified files
        for file_path in &current_files {
            if let Ok(current_metadata) = temp_monitor.get_file_metadata(file_path).await {
                if let Some(cached_metadata) = cache.get(file_path) {
                    // File exists in cache - check if modified
                    if current_metadata.modified > cached_metadata.modified ||
                       current_metadata.size != cached_metadata.size {
                        
                        let event = FileChangeEvent {
                            path: file_path.clone(),
                            change_type: FileChangeType::Modified,
                            timestamp: SystemTime::now(),
                        };
                        
                        let _ = change_sender.send(event);
                        cache.insert(file_path.clone(), current_metadata);
                    }
                } else {
                    // New file
                    let event = FileChangeEvent {
                        path: file_path.clone(),
                        change_type: FileChangeType::Created,
                        timestamp: SystemTime::now(),
                    };
                    
                    let _ = change_sender.send(event);
                    cache.insert(file_path.clone(), current_metadata);
                }
            }
        }
        
        // Check for deleted files
        let current_file_set: std::collections::HashSet<_> = 
            current_files.into_iter().collect();
        
        let deleted_files: Vec<_> = cache.keys()
            .filter(|path| !current_file_set.contains(*path))
            .cloned()
            .collect();
        
        for deleted_file in deleted_files {
            let event = FileChangeEvent {
                path: deleted_file.clone(),
                change_type: FileChangeType::Deleted,
                timestamp: SystemTime::now(),
            };
            
            let _ = change_sender.send(event);
            cache.remove(&deleted_file);
        }
        
        Ok(())
    }
    
    /// Process pending file changes
    async fn process_pending_changes(
        pending_changes: &HashMap<PathBuf, FileChangeEvent>,
        intelligence_tools: &Arc<RwLock<TuiIntelligenceTools>>,
    ) -> Result<()> {
        if pending_changes.is_empty() {
            return Ok(());
        }
        
        debug!("Processing {} pending file changes", pending_changes.len());
        
        // Group changes by type
        let mut created_files = Vec::new();
        let mut modified_files = Vec::new();
        let mut deleted_files = Vec::new();
        
        for change_event in pending_changes.values() {
            match &change_event.change_type {
                FileChangeType::Created => created_files.push(change_event.path.clone()),
                FileChangeType::Modified => modified_files.push(change_event.path.clone()),
                FileChangeType::Deleted => deleted_files.push(change_event.path.clone()),
                FileChangeType::Renamed { from, to } => {
                    deleted_files.push(from.clone());
                    created_files.push(to.clone());
                }
            }
        }
        
        // Analyze impact of changes
        let tools = intelligence_tools.read().await;
        
        if !modified_files.is_empty() || !created_files.is_empty() {
            let mut all_changed_files = modified_files.clone();
            all_changed_files.extend(created_files.clone());
            
            let file_paths: Vec<String> = all_changed_files
                .iter()
                .filter_map(|p| p.to_str().map(|s| s.to_string()))
                .collect();
            
            // Analyze impact
            let _impact = tools.analyze_change_impact(&file_paths).await;
            
            // For now, just log the changes
            info!("Detected {} file changes:", file_paths.len());
            for file in &file_paths {
                info!("  - {}", file);
            }
        }
        
        // Future: Trigger intelligence updates based on changes
        // This could include:
        // - Updating file purpose analysis
        // - Recomputing relevance scores
        // - Updating architectural understanding
        // - Triggering context refresh
        
        Ok(())
    }
}

impl Clone for FileMonitor {
    fn clone(&self) -> Self {
        let (change_sender, change_receiver) = mpsc::unbounded_channel();
        
        Self {
            project_manager: self.project_manager.clone(),
            intelligence_tools: self.intelligence_tools.clone(),
            file_cache: self.file_cache.clone(),
            change_sender,
            change_receiver: Some(change_receiver),
            debounce_duration: self.debounce_duration,
            scan_interval: self.scan_interval,
            is_running: self.is_running.clone(),
        }
    }
}

/// Helper function to start file monitoring in the background
pub async fn start_background_monitoring(
    project_manager: ProjectManager,
    intelligence_tools: TuiIntelligenceTools,
) -> Result<FileMonitor> {
    let monitor = FileMonitor::new(project_manager, intelligence_tools)?;
    
    // Start monitoring in a background task
    let monitor_clone = monitor.clone();
    tokio::spawn(async move {
        let mut background_monitor = monitor_clone;
        if let Err(e) = background_monitor.start().await {
            warn!("Background file monitoring failed: {}", e);
        }
    });
    
    Ok(monitor)
}