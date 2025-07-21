pub mod embedding;
pub mod model;
pub mod search;
pub mod config;

pub use embedding::{EmbeddingArgs, EmbeddingCommand, handle_embedding_command, quick_embedding_setup};
pub use model::{ModelArgs, ModelCommand, TaskTypeArg, handle_model_command};
pub use search::{SearchArgs, SearchCommand, PresetCommand, handle_search_command};
pub use config::{ConfigArgs, ConfigCommand, handle_config_command};