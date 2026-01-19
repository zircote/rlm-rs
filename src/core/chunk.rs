//! Chunk representation for RLM-RS.
//!
//! Chunks are segments of buffer content created by chunking strategies.
//! Each chunk maintains its position within the original buffer and
//! metadata for tracking and processing.

use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Represents a chunk of text from a buffer.
///
/// Chunks are created by chunking strategies and contain a portion of
/// buffer content along with metadata about their position and origin.
///
/// # Examples
///
/// ```
/// use rlm_rs::core::Chunk;
///
/// let chunk = Chunk::new(
///     1,
///     "Hello, world!".to_string(),
///     0..13,
///     0,
/// );
/// assert_eq!(chunk.size(), 13);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chunk {
    /// Unique identifier (assigned by storage layer).
    pub id: Option<i64>,

    /// ID of the buffer this chunk belongs to.
    pub buffer_id: i64,

    /// Chunk content.
    pub content: String,

    /// Byte range in the original buffer.
    pub byte_range: Range<usize>,

    /// Sequential index within the buffer (0-based).
    pub index: usize,

    /// Chunk metadata.
    pub metadata: ChunkMetadata,
}

/// Metadata associated with a chunk.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkMetadata {
    /// Chunking strategy that created this chunk.
    pub strategy: Option<String>,

    /// Token count estimate (if available).
    pub token_count: Option<usize>,

    /// Line range in the original buffer (if computed).
    pub line_range: Option<Range<usize>>,

    /// Unix timestamp when chunk was created.
    pub created_at: i64,

    /// Content hash for deduplication.
    pub content_hash: Option<String>,

    /// Whether this chunk overlaps with the previous chunk.
    pub has_overlap: bool,

    /// Custom metadata as JSON string.
    pub custom: Option<String>,
}

impl Chunk {
    /// Creates a new chunk.
    ///
    /// # Arguments
    ///
    /// * `buffer_id` - ID of the parent buffer.
    /// * `content` - Chunk content.
    /// * `byte_range` - Byte range in the original buffer.
    /// * `index` - Sequential index within the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use rlm_rs::core::Chunk;
    ///
    /// let chunk = Chunk::new(1, "content".to_string(), 0..7, 0);
    /// assert_eq!(chunk.buffer_id, 1);
    /// assert_eq!(chunk.index, 0);
    /// ```
    #[must_use]
    pub fn new(buffer_id: i64, content: String, byte_range: Range<usize>, index: usize) -> Self {
        Self {
            id: None,
            buffer_id,
            content,
            byte_range,
            index,
            metadata: ChunkMetadata {
                created_at: current_timestamp(),
                ..Default::default()
            },
        }
    }

    /// Creates a chunk with a specific strategy name.
    ///
    /// # Arguments
    ///
    /// * `buffer_id` - ID of the parent buffer.
    /// * `content` - Chunk content.
    /// * `byte_range` - Byte range in the original buffer.
    /// * `index` - Sequential index within the buffer.
    /// * `strategy` - Name of the chunking strategy.
    #[must_use]
    pub fn with_strategy(
        buffer_id: i64,
        content: String,
        byte_range: Range<usize>,
        index: usize,
        strategy: &str,
    ) -> Self {
        let mut chunk = Self::new(buffer_id, content, byte_range, index);
        chunk.metadata.strategy = Some(strategy.to_string());
        chunk
    }

    /// Returns the size of the chunk in bytes.
    #[must_use]
    pub fn size(&self) -> usize {
        self.content.len()
    }

    /// Returns the byte range size.
    #[must_use]
    pub const fn range_size(&self) -> usize {
        self.byte_range.end - self.byte_range.start
    }

    /// Checks if the chunk is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Returns the start byte offset in the original buffer.
    #[must_use]
    pub const fn start(&self) -> usize {
        self.byte_range.start
    }

    /// Returns the end byte offset in the original buffer.
    #[must_use]
    pub const fn end(&self) -> usize {
        self.byte_range.end
    }

    /// Sets the token count estimate.
    pub const fn set_token_count(&mut self, count: usize) {
        self.metadata.token_count = Some(count);
    }

    /// Estimates token count using a simple heuristic.
    ///
    /// Uses the approximation of ~4 characters per token.
    #[must_use]
    pub fn estimate_tokens(&self) -> usize {
        // Common approximation: ~4 chars per token
        self.content.len().div_ceil(4)
    }

    /// Sets the line range in the original buffer.
    pub const fn set_line_range(&mut self, start_line: usize, end_line: usize) {
        self.metadata.line_range = Some(start_line..end_line);
    }

    /// Marks this chunk as having overlap with the previous chunk.
    pub const fn set_has_overlap(&mut self, has_overlap: bool) {
        self.metadata.has_overlap = has_overlap;
    }

    /// Computes and sets the content hash.
    pub fn compute_hash(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.content.hash(&mut hasher);
        self.metadata.content_hash = Some(format!("{:016x}", hasher.finish()));
    }

    /// Returns a preview of the chunk content (first N characters).
    ///
    /// # Arguments
    ///
    /// * `max_len` - Maximum number of characters to include.
    #[must_use]
    pub fn preview(&self, max_len: usize) -> &str {
        if self.content.len() <= max_len {
            &self.content
        } else {
            let end = find_char_boundary(&self.content, max_len);
            &self.content[..end]
        }
    }

    /// Checks if this chunk's byte range overlaps with another range.
    #[must_use]
    pub const fn overlaps_with(&self, other_range: &Range<usize>) -> bool {
        self.byte_range.start < other_range.end && other_range.start < self.byte_range.end
    }

    /// Checks if this chunk's byte range contains a specific byte offset.
    #[must_use]
    pub fn contains_offset(&self, offset: usize) -> bool {
        self.byte_range.contains(&offset)
    }
}

/// Builder for creating chunks with fluent API.
#[derive(Debug, Default)]
pub struct ChunkBuilder {
    buffer_id: Option<i64>,
    content: Option<String>,
    byte_range: Option<Range<usize>>,
    index: Option<usize>,
    strategy: Option<String>,
    token_count: Option<usize>,
    line_range: Option<Range<usize>>,
    has_overlap: bool,
}

impl ChunkBuilder {
    /// Creates a new chunk builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the buffer ID.
    #[must_use]
    pub const fn buffer_id(mut self, id: i64) -> Self {
        self.buffer_id = Some(id);
        self
    }

    /// Sets the content.
    #[must_use]
    pub fn content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }

    /// Sets the byte range.
    #[must_use]
    pub const fn byte_range(mut self, range: Range<usize>) -> Self {
        self.byte_range = Some(range);
        self
    }

    /// Sets the index.
    #[must_use]
    pub const fn index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    /// Sets the strategy name.
    #[must_use]
    pub fn strategy(mut self, strategy: &str) -> Self {
        self.strategy = Some(strategy.to_string());
        self
    }

    /// Sets the token count.
    #[must_use]
    pub const fn token_count(mut self, count: usize) -> Self {
        self.token_count = Some(count);
        self
    }

    /// Sets the line range.
    #[must_use]
    pub const fn line_range(mut self, range: Range<usize>) -> Self {
        self.line_range = Some(range);
        self
    }

    /// Sets whether this chunk has overlap.
    #[must_use]
    pub const fn has_overlap(mut self, has_overlap: bool) -> Self {
        self.has_overlap = has_overlap;
        self
    }

    /// Builds the chunk.
    ///
    /// # Panics
    ///
    /// Panics if required fields (`buffer_id`, content, `byte_range`, index) are not set.
    #[must_use]
    pub fn build(self) -> Chunk {
        let buffer_id = self.buffer_id.unwrap_or(0);
        let content = self.content.unwrap_or_default();
        let byte_range = self.byte_range.unwrap_or(0..content.len());
        let index = self.index.unwrap_or(0);

        let mut chunk = Chunk::new(buffer_id, content, byte_range, index);

        if let Some(strategy) = self.strategy {
            chunk.metadata.strategy = Some(strategy);
        }
        if let Some(count) = self.token_count {
            chunk.metadata.token_count = Some(count);
        }
        if let Some(range) = self.line_range {
            chunk.metadata.line_range = Some(range);
        }
        chunk.metadata.has_overlap = self.has_overlap;

        chunk
    }
}

/// Finds a valid UTF-8 character boundary at or before the given position.
fn find_char_boundary(s: &str, pos: usize) -> usize {
    if pos >= s.len() {
        return s.len();
    }
    let mut boundary = pos;
    while !s.is_char_boundary(boundary) && boundary > 0 {
        boundary -= 1;
    }
    boundary
}

/// Returns the current Unix timestamp in seconds.
#[allow(clippy::cast_possible_wrap)]
fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_new() {
        let chunk = Chunk::new(1, "Hello".to_string(), 0..5, 0);
        assert_eq!(chunk.buffer_id, 1);
        assert_eq!(chunk.content, "Hello");
        assert_eq!(chunk.byte_range, 0..5);
        assert_eq!(chunk.index, 0);
        assert!(chunk.id.is_none());
    }

    #[test]
    fn test_chunk_with_strategy() {
        let chunk = Chunk::with_strategy(1, "content".to_string(), 0..7, 0, "semantic");
        assert_eq!(chunk.metadata.strategy, Some("semantic".to_string()));
    }

    #[test]
    fn test_chunk_size() {
        let chunk = Chunk::new(1, "Hello, world!".to_string(), 0..13, 0);
        assert_eq!(chunk.size(), 13);
        assert_eq!(chunk.range_size(), 13);
    }

    #[test]
    fn test_chunk_offsets() {
        let chunk = Chunk::new(1, "world".to_string(), 7..12, 1);
        assert_eq!(chunk.start(), 7);
        assert_eq!(chunk.end(), 12);
    }

    #[test]
    fn test_chunk_estimate_tokens() {
        let chunk = Chunk::new(1, "Hello, world!".to_string(), 0..13, 0);
        // 13 chars / 4 ≈ 3-4 tokens
        assert!(chunk.estimate_tokens() >= 3);
        assert!(chunk.estimate_tokens() <= 4);
    }

    #[test]
    fn test_chunk_preview() {
        let chunk = Chunk::new(1, "Hello, world!".to_string(), 0..13, 0);
        assert_eq!(chunk.preview(5), "Hello");
        assert_eq!(chunk.preview(100), "Hello, world!");
    }

    #[test]
    fn test_chunk_overlaps_with() {
        let chunk = Chunk::new(1, "test".to_string(), 10..20, 0);
        assert!(chunk.overlaps_with(&(15..25)));
        assert!(chunk.overlaps_with(&(5..15)));
        assert!(!chunk.overlaps_with(&(20..30)));
        assert!(!chunk.overlaps_with(&(0..10)));
    }

    #[test]
    fn test_chunk_contains_offset() {
        let chunk = Chunk::new(1, "test".to_string(), 10..20, 0);
        assert!(chunk.contains_offset(10));
        assert!(chunk.contains_offset(15));
        assert!(!chunk.contains_offset(20));
        assert!(!chunk.contains_offset(5));
    }

    #[test]
    fn test_chunk_hash() {
        let mut chunk1 = Chunk::new(1, "Hello".to_string(), 0..5, 0);
        let mut chunk2 = Chunk::new(2, "Hello".to_string(), 0..5, 0);
        chunk1.compute_hash();
        chunk2.compute_hash();
        assert_eq!(chunk1.metadata.content_hash, chunk2.metadata.content_hash);
    }

    #[test]
    fn test_chunk_builder() {
        let chunk = ChunkBuilder::new()
            .buffer_id(1)
            .content("test".to_string())
            .byte_range(0..4)
            .index(0)
            .strategy("fixed")
            .token_count(1)
            .line_range(0..1)
            .has_overlap(true)
            .build();

        assert_eq!(chunk.buffer_id, 1);
        assert_eq!(chunk.content, "test");
        assert_eq!(chunk.metadata.strategy, Some("fixed".to_string()));
        assert_eq!(chunk.metadata.token_count, Some(1));
        assert_eq!(chunk.metadata.line_range, Some(0..1));
        assert!(chunk.metadata.has_overlap);
    }

    #[test]
    fn test_chunk_serialization() {
        let chunk = Chunk::new(1, "test".to_string(), 0..4, 0);
        let json = serde_json::to_string(&chunk);
        assert!(json.is_ok());

        let deserialized: Result<Chunk, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.unwrap().content, "test");
    }

    #[test]
    fn test_chunk_empty() {
        let chunk = Chunk::new(1, String::new(), 0..0, 0);
        assert!(chunk.is_empty());
        assert_eq!(chunk.size(), 0);
    }
}
