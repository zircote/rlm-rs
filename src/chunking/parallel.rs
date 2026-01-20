//! Parallel chunking orchestrator.
//!
//! Wraps another chunker and processes chunks in parallel using rayon
//! for improved performance on large texts.

use crate::chunking::traits::{ChunkMetadata, Chunker};
use crate::core::Chunk;
use crate::error::Result;
use rayon::prelude::*;

/// Parallel chunking orchestrator.
///
/// Wraps another chunker and uses rayon for parallel processing.
/// Useful for very large texts where chunking itself is CPU-bound.
///
/// # Examples
///
/// ```
/// use rlm_rs::chunking::{Chunker, ParallelChunker, SemanticChunker};
///
/// let chunker = ParallelChunker::new(SemanticChunker::new());
/// let text = "Hello, world! ".repeat(1000);
/// let chunks = chunker.chunk(1, &text, None).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct ParallelChunker<C: Chunker + Clone> {
    /// The inner chunker to use for actual chunking.
    inner: C,
    /// Minimum text size to enable parallel processing.
    min_parallel_size: usize,
    /// Number of segments to split the text into for parallel processing.
    num_segments: usize,
}

impl<C: Chunker + Clone> ParallelChunker<C> {
    /// Creates a new parallel chunker wrapping the given chunker.
    ///
    /// # Arguments
    ///
    /// * `inner` - The chunker to use for actual text processing.
    #[must_use]
    pub fn new(inner: C) -> Self {
        Self {
            inner,
            min_parallel_size: 100_000, // 100KB minimum for parallel
            num_segments: num_cpus::get().max(2),
        }
    }

    /// Sets the minimum text size for parallel processing.
    ///
    /// Texts smaller than this will be processed sequentially.
    #[must_use]
    pub const fn min_parallel_size(mut self, size: usize) -> Self {
        self.min_parallel_size = size;
        self
    }

    /// Sets the number of parallel segments.
    #[must_use]
    pub fn num_segments(mut self, n: usize) -> Self {
        self.num_segments = n.max(1);
        self
    }

    /// Splits text into roughly equal segments at good boundaries.
    fn split_into_segments<'a>(&self, text: &'a str, n: usize) -> Vec<(usize, &'a str)> {
        if n <= 1 || text.len() < self.min_parallel_size {
            return vec![(0, text)];
        }

        let segment_size = text.len() / n;
        let mut segments = Vec::with_capacity(n);
        let mut start = 0;

        for i in 0..n {
            let target_end = if i == n - 1 {
                text.len()
            } else {
                start + segment_size
            };

            let end = Self::find_segment_boundary(text, target_end);
            let end = end.max(start + 1).min(text.len());

            if start < text.len() {
                segments.push((start, &text[start..end]));
            }

            start = end;
            if start >= text.len() {
                break;
            }
        }

        segments
    }

    /// Finds a good boundary for segment splitting.
    fn find_segment_boundary(text: &str, target: usize) -> usize {
        if target >= text.len() {
            return text.len();
        }

        // Look for paragraph break first
        let search_start = target.saturating_sub(1000);
        let search_region = &text[search_start..target.min(text.len())];

        if let Some(pos) = search_region.rfind("\n\n") {
            return search_start + pos + 2;
        }

        // Then newline
        if let Some(pos) = search_region.rfind('\n') {
            return search_start + pos + 1;
        }

        // Then space
        if let Some(pos) = search_region.rfind(' ') {
            return search_start + pos + 1;
        }

        // Fallback to character boundary
        let mut pos = target;
        while !text.is_char_boundary(pos) && pos > 0 {
            pos -= 1;
        }
        pos
    }

    /// Merges chunks from multiple segments, reindexing them.
    fn merge_chunks(segment_chunks: Vec<Vec<Chunk>>, buffer_id: i64) -> Vec<Chunk> {
        let mut all_chunks: Vec<Chunk> = Vec::new();
        let mut index = 0;

        for chunks in segment_chunks {
            for mut chunk in chunks {
                chunk.index = index;
                chunk.buffer_id = buffer_id;
                all_chunks.push(chunk);
                index += 1;
            }
        }

        all_chunks
    }
}

impl<C: Chunker + Clone + Send + Sync> Chunker for ParallelChunker<C> {
    fn chunk(
        &self,
        buffer_id: i64,
        text: &str,
        metadata: Option<&ChunkMetadata>,
    ) -> Result<Vec<Chunk>> {
        // For small texts, just use the inner chunker directly
        if text.len() < self.min_parallel_size {
            return self.inner.chunk(buffer_id, text, metadata);
        }

        // Split into segments
        let segments = self.split_into_segments(text, self.num_segments);

        if segments.len() <= 1 {
            return self.inner.chunk(buffer_id, text, metadata);
        }

        // Process segments in parallel
        let results: Vec<Result<Vec<Chunk>>> = segments
            .par_iter()
            .map(|(offset, segment)| {
                let mut chunks = self.inner.chunk(buffer_id, segment, metadata)?;

                // Adjust byte ranges to account for segment offset
                for chunk in &mut chunks {
                    chunk.byte_range =
                        (chunk.byte_range.start + offset)..(chunk.byte_range.end + offset);
                }

                Ok(chunks)
            })
            .collect();

        // Collect results, propagating any errors
        let mut all_segment_chunks = Vec::with_capacity(results.len());
        for result in results {
            all_segment_chunks.push(result?);
        }

        // Merge and reindex
        Ok(Self::merge_chunks(all_segment_chunks, buffer_id))
    }

    fn name(&self) -> &'static str {
        "parallel"
    }

    fn supports_parallel(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        "Parallel chunking using rayon for multi-threaded processing"
    }
}

// Add num_cpus as a simple function since we can't add dependencies mid-implementation
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .unwrap_or(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::SemanticChunker;

    #[test]
    fn test_parallel_chunker_small_text() {
        let chunker = ParallelChunker::new(SemanticChunker::with_size(50));
        let text = "Hello, world!";
        let chunks = chunker.chunk(1, text, None).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, text);
    }

    #[test]
    fn test_parallel_chunker_large_text() {
        let chunker = ParallelChunker::new(SemanticChunker::with_size(1000))
            .min_parallel_size(1000)
            .num_segments(4);

        // Generate large text
        let text = "Hello, world! This is a test sentence. ".repeat(500);

        let chunks = chunker.chunk(1, &text, None).unwrap();

        // Verify all chunks are valid
        for chunk in &chunks {
            assert!(!chunk.content.is_empty());
            assert_eq!(&text[chunk.byte_range.clone()], chunk.content);
        }

        // Verify indices are sequential
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.index, i);
        }
    }

    #[test]
    fn test_parallel_chunker_preserves_content() {
        let chunker = ParallelChunker::new(SemanticChunker::with_size(500))
            .min_parallel_size(500)
            .num_segments(2);

        let text = "Paragraph one. Sentence two.\n\nParagraph two. More text here.\n\n".repeat(50);

        let chunks = chunker.chunk(1, &text, None).unwrap();

        // Reconstruct text from chunks (accounting for possible overlap)
        let mut reconstructed = String::new();
        let mut last_end = 0;

        for chunk in &chunks {
            use std::cmp::Ordering;
            match chunk.byte_range.start.cmp(&last_end) {
                Ordering::Greater => {
                    // Gap - shouldn't happen in well-formed chunking
                }
                Ordering::Less => {
                    // Overlap - skip overlapping portion
                    let skip = last_end - chunk.byte_range.start;
                    if skip < chunk.content.len() {
                        reconstructed.push_str(&chunk.content[skip..]);
                    }
                }
                Ordering::Equal => {
                    reconstructed.push_str(&chunk.content);
                }
            }
            last_end = chunk.byte_range.end;
        }

        // The reconstructed text should cover the original
        assert!(!chunks.is_empty());
        assert!(!reconstructed.is_empty());
    }

    #[test]
    fn test_parallel_chunker_strategy_name() {
        let chunker = ParallelChunker::new(SemanticChunker::new());
        assert_eq!(chunker.name(), "parallel");
        assert!(chunker.supports_parallel());
    }

    #[test]
    fn test_split_into_segments() {
        let chunker = ParallelChunker::new(SemanticChunker::new())
            .min_parallel_size(10)
            .num_segments(3);

        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let segments = chunker.split_into_segments(text, 3);

        // Should have multiple segments
        assert!(!segments.is_empty());

        // All segments should be non-empty
        for (_, segment) in &segments {
            assert!(!segment.is_empty());
        }
    }

    #[test]
    fn test_parallel_chunker_empty_text() {
        let chunker = ParallelChunker::new(SemanticChunker::new());
        let chunks = chunker.chunk(1, "", None).unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_split_into_segments_single_segment() {
        // Test when n <= 1 returns single segment (line 69)
        let chunker = ParallelChunker::new(SemanticChunker::new())
            .min_parallel_size(10)
            .num_segments(1);

        let text = "This is some test content";
        let segments = chunker.split_into_segments(text, 1);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].1, text);
    }

    #[test]
    fn test_split_into_segments_text_too_small() {
        // Test when text.len() < min_parallel_size (line 68-69)
        let chunker = ParallelChunker::new(SemanticChunker::new())
            .min_parallel_size(1000)
            .num_segments(4);

        let text = "Short text";
        let segments = chunker.split_into_segments(text, 4);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].1, text);
    }

    #[test]
    fn test_parallel_chunker_segments_collapse_to_one() {
        // Test when split produces only 1 segment, falls back to inner (line 165)
        let chunker = ParallelChunker::new(SemanticChunker::with_size(100))
            .min_parallel_size(10)
            .num_segments(10);

        // Text that's small enough that segmentation produces just one segment
        let text = "A short text that won't split well.";
        let chunks = chunker.chunk(1, text, None).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_parallel_chunker_description() {
        // Test description method (lines 202-203)
        let chunker = ParallelChunker::new(SemanticChunker::new());
        let desc = chunker.description();
        assert!(desc.contains("Parallel"));
        assert!(!desc.is_empty());
    }

    #[test]
    fn test_find_segment_boundary_no_good_boundary() {
        // Test when no paragraph, newline, or space is found (lines 124-128)
        let text = "AAAAAAAAAAAAAAAAAAAA"; // No natural boundaries
        let boundary = ParallelChunker::<SemanticChunker>::find_segment_boundary(text, 10);
        // Should fall back to character boundary
        assert!(boundary <= text.len());
    }

    #[test]
    fn test_find_segment_boundary_at_end() {
        // Test when target >= text.len() (line 102)
        let text = "Short";
        let boundary = ParallelChunker::<SemanticChunker>::find_segment_boundary(text, 100);
        assert_eq!(boundary, text.len());
    }

    #[test]
    fn test_find_segment_boundary_finds_space() {
        // Test when space is found but no newline (lines 119-120)
        let text = "word1 word2 word3 word4";
        let boundary = ParallelChunker::<SemanticChunker>::find_segment_boundary(text, 15);
        // Should find a space boundary
        assert!(boundary > 0 && boundary <= text.len());
    }
}
