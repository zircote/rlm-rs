//! Chunking strategies for RLM-RS.
//!
//! This module provides a trait-based system for chunking text content
//! into processable segments. Multiple strategies are available:
//!
//! - **Fixed**: Simple character-based chunking with configurable size and overlap
//! - **Semantic**: Unicode-aware chunking respecting sentence/paragraph boundaries
//! - **Parallel**: Orchestrator for parallel chunk processing

pub mod fixed;
pub mod parallel;
pub mod semantic;
pub mod traits;

pub use fixed::FixedChunker;
pub use parallel::ParallelChunker;
pub use semantic::SemanticChunker;
pub use traits::{ChunkMetadata as ChunkerMetadata, Chunker};

/// Default chunk size in characters (~10k tokens at 4 chars/token).
/// Sized to fit comfortably within Claude's 25k token read limit.
pub const DEFAULT_CHUNK_SIZE: usize = 40_000;

/// Default overlap size in characters (for context continuity).
pub const DEFAULT_OVERLAP: usize = 500;

/// Maximum allowed chunk size (250k chars, ~62k tokens).
pub const MAX_CHUNK_SIZE: usize = 250_000;

/// Creates the default chunker (semantic).
#[must_use]
pub const fn default_chunker() -> SemanticChunker {
    SemanticChunker::new()
}

/// Creates a chunker by name.
///
/// # Arguments
///
/// * `name` - Chunker strategy name: "fixed", "semantic", or "parallel".
///
/// # Returns
///
/// A boxed chunker trait object, or an error for unknown strategies.
///
/// # Errors
///
/// Returns [`crate::error::ChunkingError::UnknownStrategy`] if the strategy name is not recognized.
pub fn create_chunker(name: &str) -> crate::error::Result<Box<dyn Chunker>> {
    match name.to_lowercase().as_str() {
        "fixed" => Ok(Box::new(FixedChunker::new())),
        "semantic" => Ok(Box::new(SemanticChunker::new())),
        "parallel" => Ok(Box::new(ParallelChunker::new(SemanticChunker::new()))),
        _ => Err(crate::error::ChunkingError::UnknownStrategy {
            name: name.to_string(),
        }
        .into()),
    }
}

/// Lists available chunking strategy names.
#[must_use]
pub fn available_strategies() -> Vec<&'static str> {
    vec!["fixed", "semantic", "parallel"]
}
