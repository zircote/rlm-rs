//! Fixed-size chunking strategy.
//!
//! Provides simple character-based chunking with configurable size and overlap.
//! Respects UTF-8 character boundaries to avoid splitting multi-byte characters.

use crate::chunking::traits::{ChunkMetadata, Chunker};
use crate::chunking::{DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP, MAX_CHUNK_SIZE};
use crate::core::Chunk;
use crate::error::{ChunkingError, Result};

/// Fixed-size chunker that splits text at character boundaries.
///
/// This is the simplest chunking strategy, splitting text into
/// fixed-size segments with optional overlap. It ensures chunks
/// never split multi-byte UTF-8 characters.
///
/// # Examples
///
/// ```
/// use rlm_rs::chunking::{Chunker, FixedChunker};
///
/// let chunker = FixedChunker::with_size(100);
/// let text = "Hello, world! ".repeat(20);
/// let chunks = chunker.chunk(1, &text, None).unwrap();
/// for chunk in &chunks {
///     assert!(chunk.size() <= 100);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct FixedChunker {
    /// Target chunk size in characters.
    chunk_size: usize,
    /// Overlap between consecutive chunks.
    overlap: usize,
    /// Whether to align chunks to line boundaries.
    line_aware: bool,
}

impl Default for FixedChunker {
    fn default() -> Self {
        Self::new()
    }
}

impl FixedChunker {
    /// Creates a new fixed chunker with default settings.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            overlap: DEFAULT_OVERLAP,
            line_aware: true,
        }
    }

    /// Creates a fixed chunker with custom chunk size and no overlap.
    ///
    /// # Arguments
    ///
    /// * `chunk_size` - Target size for each chunk in characters.
    #[must_use]
    pub const fn with_size(chunk_size: usize) -> Self {
        Self {
            chunk_size,
            overlap: 0,
            line_aware: true,
        }
    }

    /// Creates a fixed chunker with custom size and overlap.
    ///
    /// # Arguments
    ///
    /// * `chunk_size` - Target size for each chunk in characters.
    /// * `overlap` - Number of characters to overlap between chunks.
    #[must_use]
    pub const fn with_size_and_overlap(chunk_size: usize, overlap: usize) -> Self {
        Self {
            chunk_size,
            overlap,
            line_aware: true,
        }
    }

    /// Sets whether to align chunks to line boundaries.
    ///
    /// When enabled, chunks will end at the nearest newline before
    /// the target size (if one exists within a reasonable range).
    #[must_use]
    pub const fn line_aware(mut self, enabled: bool) -> Self {
        self.line_aware = enabled;
        self
    }

    /// Finds a valid chunk boundary respecting UTF-8 and optionally lines.
    fn find_boundary(&self, text: &str, target_pos: usize) -> usize {
        let mut pos = target_pos.min(text.len());

        // First, find valid UTF-8 boundary
        while !text.is_char_boundary(pos) && pos > 0 {
            pos -= 1;
        }

        // If line-aware, try to find a newline before this position
        if self.line_aware && pos > 0 {
            let search_start = pos.saturating_sub(self.chunk_size / 10); // Look back up to 10%
            if let Some(newline_offset) = text[search_start..pos].rfind('\n') {
                let newline_pos = search_start + newline_offset + 1; // Position after newline
                if newline_pos > search_start {
                    return newline_pos;
                }
            }
        }

        pos
    }
}

impl Chunker for FixedChunker {
    fn chunk(
        &self,
        buffer_id: i64,
        text: &str,
        metadata: Option<&ChunkMetadata>,
    ) -> Result<Vec<Chunk>> {
        // Get effective chunk size and overlap
        let (chunk_size, overlap) = metadata.map_or((self.chunk_size, self.overlap), |meta| {
            (meta.chunk_size, meta.overlap)
        });

        // Validate configuration
        if chunk_size == 0 {
            return Err(ChunkingError::InvalidConfig {
                reason: "chunk_size must be > 0".to_string(),
            }
            .into());
        }
        if chunk_size > MAX_CHUNK_SIZE {
            return Err(ChunkingError::ChunkTooLarge {
                size: chunk_size,
                max: MAX_CHUNK_SIZE,
            }
            .into());
        }
        if overlap >= chunk_size {
            return Err(ChunkingError::OverlapTooLarge {
                overlap,
                size: chunk_size,
            }
            .into());
        }

        // Handle empty text
        if text.is_empty() {
            return Ok(vec![]);
        }

        // Handle text smaller than chunk size
        if text.len() <= chunk_size {
            return Ok(vec![Chunk::with_strategy(
                buffer_id,
                text.to_string(),
                0..text.len(),
                0,
                self.name(),
            )]);
        }

        let mut chunks = Vec::new();
        let mut start = 0;
        let mut index = 0;

        while start < text.len() {
            let target_end = (start + chunk_size).min(text.len());
            let end = if target_end >= text.len() {
                text.len()
            } else {
                self.find_boundary(text, target_end)
            };

            // Ensure we make progress
            let end = if end <= start {
                (start + chunk_size).min(text.len())
            } else {
                end
            };

            let content = text[start..end].to_string();
            let mut chunk =
                Chunk::with_strategy(buffer_id, content, start..end, index, self.name());

            if index > 0 && overlap > 0 {
                chunk.set_has_overlap(true);
            }

            chunks.push(chunk);

            // Check max chunks limit
            if let Some(meta) = metadata {
                if meta.max_chunks > 0 && chunks.len() >= meta.max_chunks {
                    break;
                }
            }

            // Move to next chunk
            if end >= text.len() {
                break;
            }

            start = if overlap > 0 {
                end.saturating_sub(overlap)
            } else {
                end
            };

            // Ensure we don't go backwards
            if start <= chunks.last().map_or(0, |c| c.byte_range.start) {
                start = end;
            }

            index += 1;
        }

        Ok(chunks)
    }

    fn name(&self) -> &'static str {
        "fixed"
    }

    fn description(&self) -> &'static str {
        "Fixed-size chunking with optional line boundary alignment"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_chunker_default() {
        let chunker = FixedChunker::new();
        assert_eq!(chunker.chunk_size, DEFAULT_CHUNK_SIZE);
        assert_eq!(chunker.overlap, DEFAULT_OVERLAP);
    }

    #[test]
    fn test_fixed_chunker_empty_text() {
        let chunker = FixedChunker::with_size(100);
        let chunks = chunker.chunk(1, "", None).unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_fixed_chunker_small_text() {
        let chunker = FixedChunker::with_size(100);
        let text = "Hello, world!";
        let chunks = chunker.chunk(1, text, None).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, text);
    }

    #[test]
    fn test_fixed_chunker_exact_size() {
        let chunker = FixedChunker::with_size(10).line_aware(false);
        let text = "0123456789";
        let chunks = chunker.chunk(1, text, None).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, text);
    }

    #[test]
    fn test_fixed_chunker_multiple_chunks() {
        let chunker = FixedChunker::with_size(10).line_aware(false);
        let text = "0123456789ABCDEFGHIJ";
        let chunks = chunker.chunk(1, text, None).unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].byte_range, 0..10);
        assert_eq!(chunks[1].byte_range, 10..20);
    }

    #[test]
    fn test_fixed_chunker_with_overlap() {
        let chunker = FixedChunker::with_size_and_overlap(10, 3).line_aware(false);
        let text = "0123456789ABCDEFGHIJ";
        let chunks = chunker.chunk(1, text, None).unwrap();

        // With overlap, second chunk should start at 7 (10 - 3)
        assert!(chunks.len() >= 2);
        assert!(chunks[1].metadata.has_overlap);
    }

    #[test]
    fn test_fixed_chunker_line_aware() {
        let chunker = FixedChunker::with_size(15).line_aware(true);
        let text = "Hello\nWorld\nTest";
        let chunks = chunker.chunk(1, text, None).unwrap();

        // Should try to align to newline
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_fixed_chunker_unicode() {
        let chunker = FixedChunker::with_size(5).line_aware(false);
        let text = "Hello世界Test";
        let chunks = chunker.chunk(1, text, None).unwrap();

        // All chunks should be valid UTF-8
        for chunk in &chunks {
            assert!(chunk.content.is_char_boundary(0));
        }
    }

    #[test]
    fn test_fixed_chunker_preserves_indices() {
        let chunker = FixedChunker::with_size(10).line_aware(false);
        let text = "0123456789ABCDEFGHIJ";
        let chunks = chunker.chunk(1, text, None).unwrap();

        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.index, i);
            assert_eq!(chunk.buffer_id, 1);
        }
    }

    #[test]
    fn test_fixed_chunker_invalid_config() {
        let chunker = FixedChunker::with_size(0);
        let result = chunker.chunk(1, "test", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_fixed_chunker_overlap_too_large() {
        let chunker = FixedChunker::with_size_and_overlap(10, 10);
        let result = chunker.chunk(1, "test content here", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_fixed_chunker_max_chunks() {
        let chunker = FixedChunker::with_size(5).line_aware(false);
        let text = "0123456789ABCDEFGHIJ";
        let meta = ChunkMetadata::with_size(5).max_chunks(2);
        let chunks = chunker.chunk(1, text, Some(&meta)).unwrap();
        assert_eq!(chunks.len(), 2);
    }

    #[test]
    fn test_fixed_chunker_strategy_name() {
        let chunker = FixedChunker::new();
        assert_eq!(chunker.name(), "fixed");

        let chunks = chunker.chunk(1, "Hello, world!", None).unwrap();
        assert_eq!(chunks[0].metadata.strategy, Some("fixed".to_string()));
    }
}
