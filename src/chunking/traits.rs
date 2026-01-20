//! Chunker trait definition.
//!
//! Defines the interface for all chunking strategies, enabling
//! pluggable text segmentation approaches.

use crate::core::Chunk;
use crate::error::Result;

/// Trait for chunking text into processable segments.
///
/// Implementations must be `Send + Sync` to support parallel processing.
/// Each chunker should produce consistent, deterministic output for the
/// same input.
///
/// # Examples
///
/// ```
/// use rlm_rs::chunking::{Chunker, FixedChunker};
///
/// let chunker = FixedChunker::with_size(100);
/// let text = "Hello, world! ".repeat(20);
/// let chunks = chunker.chunk(1, &text, None).unwrap();
/// assert!(!chunks.is_empty());
/// ```
pub trait Chunker: Send + Sync {
    /// Chunks the input text into segments.
    ///
    /// # Arguments
    ///
    /// * `buffer_id` - ID of the source buffer.
    /// * `text` - The input text to chunk.
    /// * `metadata` - Optional metadata for context-aware chunking.
    ///
    /// # Returns
    ///
    /// A vector of chunks with byte offsets and metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if chunking fails (e.g., invalid configuration).
    fn chunk(
        &self,
        buffer_id: i64,
        text: &str,
        metadata: Option<&ChunkMetadata>,
    ) -> Result<Vec<Chunk>>;

    /// Returns the name of the chunking strategy.
    fn name(&self) -> &'static str;

    /// Returns whether this chunker supports parallel processing.
    ///
    /// Default is `false`. Chunkers that benefit from parallelization
    /// should override this to return `true`.
    fn supports_parallel(&self) -> bool {
        false
    }

    /// Returns a description of the chunking strategy.
    fn description(&self) -> &'static str {
        "No description available"
    }

    /// Validates configuration before chunking.
    ///
    /// # Arguments
    ///
    /// * `metadata` - Optional metadata to validate.
    ///
    /// # Returns
    ///
    /// `Ok(())` if configuration is valid, error otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if chunk size is zero or overlap exceeds chunk size.
    fn validate(&self, metadata: Option<&ChunkMetadata>) -> Result<()> {
        if let Some(meta) = metadata {
            if meta.chunk_size == 0 {
                return Err(crate::error::ChunkingError::InvalidConfig {
                    reason: "chunk_size must be > 0".to_string(),
                }
                .into());
            }
            if meta.overlap >= meta.chunk_size {
                return Err(crate::error::ChunkingError::OverlapTooLarge {
                    overlap: meta.overlap,
                    size: meta.chunk_size,
                }
                .into());
            }
        }
        Ok(())
    }
}

/// Metadata provided to chunkers for context-aware processing.
///
/// This allows callers to customize chunking behavior without
/// modifying the chunker itself.
#[derive(Debug, Clone, Default)]
pub struct ChunkMetadata {
    /// Source file path (for content-type detection).
    pub source: Option<String>,

    /// File MIME type or extension (e.g., "md", "json", "py").
    pub content_type: Option<String>,

    /// Target chunk size in characters.
    pub chunk_size: usize,

    /// Overlap between consecutive chunks.
    pub overlap: usize,

    /// Whether to preserve line boundaries.
    pub preserve_lines: bool,

    /// Whether to preserve sentence boundaries.
    pub preserve_sentences: bool,

    /// Maximum chunks to produce (0 = unlimited).
    pub max_chunks: usize,
}

impl ChunkMetadata {
    /// Creates new metadata with default chunk size.
    #[must_use]
    pub fn new() -> Self {
        Self {
            chunk_size: super::DEFAULT_CHUNK_SIZE,
            overlap: super::DEFAULT_OVERLAP,
            preserve_lines: true,
            preserve_sentences: false,
            ..Default::default()
        }
    }

    /// Creates metadata with custom chunk size and no overlap.
    #[must_use]
    pub fn with_size(chunk_size: usize) -> Self {
        Self {
            chunk_size,
            overlap: 0,
            ..Self::new()
        }
    }

    /// Creates metadata with custom size and overlap.
    #[must_use]
    pub fn with_size_and_overlap(chunk_size: usize, overlap: usize) -> Self {
        Self {
            chunk_size,
            overlap,
            ..Self::new()
        }
    }

    /// Sets the source path.
    #[must_use]
    pub fn source(mut self, source: &str) -> Self {
        self.source = Some(source.to_string());
        self
    }

    /// Sets the content type.
    #[must_use]
    pub fn content_type(mut self, content_type: &str) -> Self {
        self.content_type = Some(content_type.to_string());
        self
    }

    /// Sets whether to preserve line boundaries.
    #[must_use]
    pub const fn preserve_lines(mut self, preserve: bool) -> Self {
        self.preserve_lines = preserve;
        self
    }

    /// Sets whether to preserve sentence boundaries.
    #[must_use]
    pub const fn preserve_sentences(mut self, preserve: bool) -> Self {
        self.preserve_sentences = preserve;
        self
    }

    /// Sets maximum chunks.
    #[must_use]
    pub const fn max_chunks(mut self, max: usize) -> Self {
        self.max_chunks = max;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_metadata_defaults() {
        let meta = ChunkMetadata::new();
        assert_eq!(meta.chunk_size, super::super::DEFAULT_CHUNK_SIZE);
        assert_eq!(meta.overlap, super::super::DEFAULT_OVERLAP);
        assert!(meta.preserve_lines);
        assert!(!meta.preserve_sentences);
    }

    #[test]
    fn test_chunk_metadata_builder() {
        let meta = ChunkMetadata::with_size_and_overlap(1000, 100)
            .source("test.txt")
            .content_type("txt")
            .preserve_sentences(true)
            .max_chunks(10);

        assert_eq!(meta.chunk_size, 1000);
        assert_eq!(meta.overlap, 100);
        assert_eq!(meta.source, Some("test.txt".to_string()));
        assert_eq!(meta.content_type, Some("txt".to_string()));
        assert!(meta.preserve_sentences);
        assert_eq!(meta.max_chunks, 10);
    }

    #[test]
    fn test_chunk_metadata_with_size() {
        let meta = ChunkMetadata::with_size(500);
        assert_eq!(meta.chunk_size, 500);
        assert_eq!(meta.overlap, 0);
    }

    #[test]
    fn test_chunk_metadata_preserve_lines() {
        let meta = ChunkMetadata::new().preserve_lines(false);
        assert!(!meta.preserve_lines);

        let meta = ChunkMetadata::new().preserve_lines(true);
        assert!(meta.preserve_lines);
    }

    // Test validation through FixedChunker since trait methods need a concrete impl
    mod validation_tests {
        use crate::chunking::FixedChunker;
        use crate::chunking::traits::{ChunkMetadata, Chunker};

        #[test]
        fn test_chunker_validate_zero_chunk_size() {
            let chunker = FixedChunker::with_size(100);
            let meta = ChunkMetadata {
                chunk_size: 0,
                overlap: 0,
                ..Default::default()
            };
            let result = chunker.validate(Some(&meta));
            assert!(result.is_err());
        }

        #[test]
        fn test_chunker_validate_overlap_too_large() {
            let chunker = FixedChunker::with_size(100);
            let meta = ChunkMetadata {
                chunk_size: 50,
                overlap: 100, // overlap >= chunk_size
                ..Default::default()
            };
            let result = chunker.validate(Some(&meta));
            assert!(result.is_err());
        }

        #[test]
        fn test_chunker_validate_valid() {
            let chunker = FixedChunker::with_size(100);
            let meta = ChunkMetadata {
                chunk_size: 100,
                overlap: 10,
                ..Default::default()
            };
            let result = chunker.validate(Some(&meta));
            assert!(result.is_ok());
        }

        #[test]
        fn test_chunker_validate_none() {
            let chunker = FixedChunker::with_size(100);
            let result = chunker.validate(None);
            assert!(result.is_ok());
        }

        #[test]
        fn test_chunker_supports_parallel() {
            let chunker = FixedChunker::with_size(100);
            // FixedChunker doesn't support parallel by default
            assert!(!chunker.supports_parallel());
        }

        #[test]
        fn test_chunker_description() {
            let chunker = FixedChunker::with_size(100);
            let desc = chunker.description();
            assert!(!desc.is_empty());
        }

        #[test]
        fn test_chunker_name() {
            let chunker = FixedChunker::with_size(100);
            assert_eq!(chunker.name(), "fixed");
        }
    }

    /// A minimal chunker that uses all default trait implementations
    struct MinimalChunker;

    impl Chunker for MinimalChunker {
        fn chunk(
            &self,
            _buffer_id: i64,
            _text: &str,
            _metadata: Option<&ChunkMetadata>,
        ) -> crate::error::Result<Vec<crate::core::Chunk>> {
            Ok(vec![])
        }

        fn name(&self) -> &'static str {
            "minimal"
        }
    }

    #[test]
    fn test_chunker_default_description() {
        // Test default description() method (lines 60-61)
        let chunker = MinimalChunker;
        let desc = chunker.description();
        assert_eq!(desc, "No description available");
    }

    #[test]
    fn test_chunker_default_supports_parallel() {
        // Test default supports_parallel() (line 56)
        let chunker = MinimalChunker;
        assert!(!chunker.supports_parallel());
    }
}
