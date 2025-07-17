pub mod embedding;
pub mod search;

pub use embedding::{EmbeddingArgs, EmbeddingCommand, handle_embedding_command, quick_embedding_setup};
pub use search::{SearchArgs, SearchCommand, handle_search_command};