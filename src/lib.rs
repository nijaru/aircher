//! Aircher - Intelligent coding agent
#![allow(dead_code, unused_variables)]

pub mod agent;
pub mod app;
pub mod auth;
pub mod client;
pub mod cli;
pub mod commands;
pub mod config;
pub mod context;
pub mod conversation;
pub mod cost;
pub mod intelligence;
pub mod permissions;
pub mod project;
pub mod providers;
pub mod semantic_search;
pub mod vector_search;
pub mod code_chunking;
pub mod search_presets;
pub mod search_display;
pub mod query_intelligence;
pub mod server;
pub mod sessions;
pub mod storage;
pub mod ui;
pub mod utils;

#[cfg(any(test, feature = "testing"))]
pub mod testing;

#[cfg(any(test, feature = "testing"))]
pub mod benchmarks;

#[cfg(feature = "mcp")]
pub mod mcp;