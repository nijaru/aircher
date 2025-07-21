pub mod app;
pub mod cli;
pub mod commands;
pub mod config;
pub mod context;
pub mod cost;
pub mod intelligence;
pub mod project;
pub mod providers;
pub mod semantic_search;
pub mod vector_search;
pub mod code_chunking;
pub mod search_presets;
pub mod search_display;
pub mod sessions;
pub mod storage;
pub mod ui;
pub mod utils;

#[cfg(any(test, feature = "testing"))]
pub mod testing;