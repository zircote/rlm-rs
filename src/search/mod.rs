//! Hybrid search with semantic and lexical retrieval.
//!
//! Combines vector similarity search with FTS5 BM25 using Reciprocal Rank Fusion (RRF).

mod rrf;

pub use rrf::{RrfConfig, reciprocal_rank_fusion, weighted_rrf};

use crate::embedding::{Embedder, cosine_similarity};
use crate::error::Result;
use crate::storage::{SqliteStorage, Storage};

/// Default similarity threshold for semantic search.
pub const DEFAULT_SIMILARITY_THRESHOLD: f32 = 0.3;

/// Default number of results to return.
pub const DEFAULT_TOP_K: usize = 10;

/// Search result with chunk ID and combined score.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Chunk ID.
    pub chunk_id: i64,
    /// Buffer ID this chunk belongs to.
    pub buffer_id: i64,
    /// Sequential index within the buffer (0-based, for temporal ordering).
    pub index: usize,
    /// Combined RRF score (higher is better).
    pub score: f64,
    /// Semantic similarity score (if available).
    pub semantic_score: Option<f32>,
    /// BM25 score (if available).
    pub bm25_score: Option<f64>,
}

/// Configuration for hybrid search.
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Maximum number of results to return.
    pub top_k: usize,
    /// Minimum similarity threshold for semantic results.
    pub similarity_threshold: f32,
    /// RRF k parameter (default 60).
    pub rrf_k: u32,
    /// Whether to include semantic search.
    pub use_semantic: bool,
    /// Whether to include BM25 search.
    pub use_bm25: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            top_k: DEFAULT_TOP_K,
            similarity_threshold: DEFAULT_SIMILARITY_THRESHOLD,
            rrf_k: 60,
            use_semantic: true,
            use_bm25: true,
        }
    }
}

impl SearchResult {
    /// Creates a new search result, looking up chunk metadata from storage.
    ///
    /// Returns `None` if the chunk cannot be found.
    fn from_chunk_id(
        storage: &SqliteStorage,
        chunk_id: i64,
        score: f64,
        semantic_score: Option<f32>,
        bm25_score: Option<f64>,
    ) -> Option<Self> {
        storage
            .get_chunk(chunk_id)
            .ok()
            .flatten()
            .map(|chunk| Self {
                chunk_id,
                buffer_id: chunk.buffer_id,
                index: chunk.index,
                score,
                semantic_score,
                bm25_score,
            })
    }
}

impl SearchConfig {
    /// Creates a new search config with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the top-k limit.
    #[must_use]
    pub const fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = top_k;
        self
    }

    /// Sets the similarity threshold.
    #[must_use]
    pub const fn with_threshold(mut self, threshold: f32) -> Self {
        self.similarity_threshold = threshold;
        self
    }

    /// Sets the RRF k parameter.
    #[must_use]
    pub const fn with_rrf_k(mut self, k: u32) -> Self {
        self.rrf_k = k;
        self
    }

    /// Enables or disables semantic search.
    #[must_use]
    pub const fn with_semantic(mut self, enabled: bool) -> Self {
        self.use_semantic = enabled;
        self
    }

    /// Enables or disables BM25 search.
    #[must_use]
    pub const fn with_bm25(mut self, enabled: bool) -> Self {
        self.use_bm25 = enabled;
        self
    }
}

/// Performs hybrid search combining semantic and BM25 results.
///
/// # Arguments
///
/// * `storage` - The storage backend.
/// * `embedder` - The embedding generator.
/// * `query` - The search query text.
/// * `config` - Search configuration.
///
/// # Errors
///
/// Returns an error if search operations fail.
pub fn hybrid_search(
    storage: &SqliteStorage,
    embedder: &dyn Embedder,
    query: &str,
    config: &SearchConfig,
) -> Result<Vec<SearchResult>> {
    let mut semantic_results: Vec<(i64, f32)> = Vec::new();
    let mut bm25_results: Vec<(i64, f64)> = Vec::new();

    // Semantic search
    if config.use_semantic {
        semantic_results = semantic_search(storage, embedder, query, config)?;
    }

    // BM25 search
    if config.use_bm25 {
        bm25_results = storage.search_fts(query, config.top_k * 2)?;
    }

    // If only one type of search is enabled, return those results directly
    if !config.use_semantic {
        return Ok(bm25_results
            .into_iter()
            .take(config.top_k)
            .filter_map(|(chunk_id, score)| {
                SearchResult::from_chunk_id(storage, chunk_id, score, None, Some(score))
            })
            .collect());
    }

    if !config.use_bm25 {
        return Ok(semantic_results
            .into_iter()
            .take(config.top_k)
            .filter_map(|(chunk_id, score)| {
                SearchResult::from_chunk_id(storage, chunk_id, f64::from(score), Some(score), None)
            })
            .collect());
    }

    // Combine using RRF
    let rrf_config = RrfConfig::new(config.rrf_k);

    // Convert to ranked lists (already sorted by score descending)
    let semantic_ranked: Vec<i64> = semantic_results.iter().map(|(id, _)| *id).collect();
    let bm25_ranked: Vec<i64> = bm25_results.iter().map(|(id, _)| *id).collect();

    let fused = reciprocal_rank_fusion(&[&semantic_ranked, &bm25_ranked], &rrf_config);

    // Build result with original scores
    let semantic_map: std::collections::HashMap<i64, f32> = semantic_results.into_iter().collect();
    let bm25_map: std::collections::HashMap<i64, f64> = bm25_results.into_iter().collect();

    let results: Vec<SearchResult> = fused
        .into_iter()
        .take(config.top_k)
        .filter_map(|(chunk_id, rrf_score)| {
            SearchResult::from_chunk_id(
                storage,
                chunk_id,
                rrf_score,
                semantic_map.get(&chunk_id).copied(),
                bm25_map.get(&chunk_id).copied(),
            )
        })
        .collect();

    Ok(results)
}

/// Performs semantic similarity search.
///
/// Uses cosine similarity between query embedding and stored chunk embeddings.
fn semantic_search(
    storage: &SqliteStorage,
    embedder: &dyn Embedder,
    query: &str,
    config: &SearchConfig,
) -> Result<Vec<(i64, f32)>> {
    // Generate query embedding
    let query_embedding = embedder.embed(query)?;

    // Get all embeddings from storage
    let all_embeddings = storage.get_all_embeddings()?;

    if all_embeddings.is_empty() {
        return Ok(Vec::new());
    }

    // Calculate similarities
    let mut similarities: Vec<(i64, f32)> = all_embeddings
        .iter()
        .map(|(chunk_id, embedding)| {
            let sim = cosine_similarity(&query_embedding, embedding);
            (*chunk_id, sim)
        })
        .filter(|(_, sim)| *sim >= config.similarity_threshold)
        .collect();

    // Sort by similarity descending
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Limit results
    similarities.truncate(config.top_k * 2);

    Ok(similarities)
}

/// Performs semantic-only search.
///
/// # Arguments
///
/// * `storage` - The storage backend.
/// * `embedder` - The embedding generator.
/// * `query` - The search query text.
/// * `top_k` - Maximum number of results.
/// * `threshold` - Minimum similarity threshold.
///
/// # Errors
///
/// Returns an error if search fails.
pub fn search_semantic(
    storage: &SqliteStorage,
    embedder: &dyn Embedder,
    query: &str,
    top_k: usize,
    threshold: f32,
) -> Result<Vec<SearchResult>> {
    let config = SearchConfig::new()
        .with_top_k(top_k)
        .with_threshold(threshold)
        .with_semantic(true)
        .with_bm25(false);

    hybrid_search(storage, embedder, query, &config)
}

/// Performs BM25-only search.
///
/// # Arguments
///
/// * `storage` - The storage backend.
/// * `query` - The search query text.
/// * `top_k` - Maximum number of results.
///
/// # Errors
///
/// Returns an error if search fails.
pub fn search_bm25(
    storage: &SqliteStorage,
    query: &str,
    top_k: usize,
) -> Result<Vec<SearchResult>> {
    let results = storage.search_fts(query, top_k)?;

    Ok(results
        .into_iter()
        .filter_map(|(chunk_id, score)| {
            SearchResult::from_chunk_id(storage, chunk_id, score, None, Some(score))
        })
        .collect())
}

/// Generates and stores embeddings for all chunks in a buffer.
///
/// # Arguments
///
/// * `storage` - The storage backend (mutable for storing embeddings).
/// * `embedder` - The embedding generator.
/// * `buffer_id` - The buffer ID to process.
///
/// # Returns
///
/// The number of chunks embedded.
///
/// # Errors
///
/// Returns an error if embedding generation or storage fails.
pub fn embed_buffer_chunks(
    storage: &mut SqliteStorage,
    embedder: &dyn Embedder,
    buffer_id: i64,
) -> Result<usize> {
    let chunks = storage.get_chunks(buffer_id)?;

    if chunks.is_empty() {
        return Ok(0);
    }

    // Collect chunk texts for batch embedding
    let texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();

    // Generate embeddings in batch
    let embeddings = embedder.embed_batch(&texts)?;

    // Prepare batch for storage
    let batch: Vec<(i64, Vec<f32>)> = chunks
        .iter()
        .zip(embeddings)
        .filter_map(|(chunk, embedding)| chunk.id.map(|id| (id, embedding)))
        .collect();

    let count = batch.len();

    // Store embeddings
    storage.store_embeddings_batch(&batch, None)?;

    Ok(count)
}

/// Checks if a buffer has all chunks embedded.
///
/// # Errors
///
/// Returns an error if the check fails.
pub fn buffer_fully_embedded(storage: &SqliteStorage, buffer_id: i64) -> Result<bool> {
    let chunk_count = storage.chunk_count(buffer_id)?;
    if chunk_count == 0 {
        return Ok(true);
    }

    // Count embeddings for this buffer's chunks
    let chunks = storage.get_chunks(buffer_id)?;
    let mut embedded_count = 0;

    for chunk in &chunks {
        if let Some(id) = chunk.id
            && storage.has_embedding(id)?
        {
            embedded_count += 1;
        }
    }

    Ok(embedded_count == chunk_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Buffer, Chunk};
    use crate::embedding::{DEFAULT_DIMENSIONS, FallbackEmbedder};
    use crate::storage::Storage;

    fn setup_storage() -> SqliteStorage {
        let mut storage = SqliteStorage::in_memory().unwrap();
        storage.init().unwrap();
        storage
    }

    fn setup_storage_with_chunks() -> SqliteStorage {
        let mut storage = setup_storage();

        // Create a buffer
        let buffer = Buffer::from_named(
            "test.txt".to_string(),
            "Test content for searching".to_string(),
        );
        let buffer_id = storage.add_buffer(&buffer).unwrap();

        // Create chunks with different content
        let chunks = vec![
            Chunk::new(
                buffer_id,
                "The quick brown fox jumps over the lazy dog".to_string(),
                0..44,
                0,
            ),
            Chunk::new(
                buffer_id,
                "Machine learning is a subset of artificial intelligence".to_string(),
                44..100,
                1,
            ),
            Chunk::new(
                buffer_id,
                "Rust is a systems programming language".to_string(),
                100..139,
                2,
            ),
        ];

        storage.add_chunks(buffer_id, &chunks).unwrap();

        storage
    }

    #[test]
    fn test_search_config_default() {
        let config = SearchConfig::default();
        assert_eq!(config.top_k, DEFAULT_TOP_K);
        assert!((config.similarity_threshold - DEFAULT_SIMILARITY_THRESHOLD).abs() < f32::EPSILON);
        assert_eq!(config.rrf_k, 60);
        assert!(config.use_semantic);
        assert!(config.use_bm25);
    }

    #[test]
    fn test_search_config_builder() {
        let config = SearchConfig::new()
            .with_top_k(20)
            .with_threshold(0.5)
            .with_rrf_k(30)
            .with_semantic(false)
            .with_bm25(true);

        assert_eq!(config.top_k, 20);
        assert!((config.similarity_threshold - 0.5).abs() < f32::EPSILON);
        assert_eq!(config.rrf_k, 30);
        assert!(!config.use_semantic);
        assert!(config.use_bm25);
    }

    #[test]
    fn test_search_bm25() {
        let storage = setup_storage_with_chunks();

        // Search for "fox" - should find the first chunk
        let results = search_bm25(&storage, "fox", 10).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].bm25_score.is_some());
        assert!(results[0].semantic_score.is_none());
    }

    #[test]
    fn test_search_bm25_no_results() {
        let storage = setup_storage_with_chunks();

        // Search for something not in the content
        let results = search_bm25(&storage, "xyz123nonexistent", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_embed_buffer_chunks() {
        let mut storage = setup_storage_with_chunks();
        let embedder = FallbackEmbedder::new(DEFAULT_DIMENSIONS);

        // Embed chunks for buffer 1
        let count = embed_buffer_chunks(&mut storage, &embedder, 1).unwrap();
        assert_eq!(count, 3); // We created 3 chunks
    }

    #[test]
    fn test_embed_buffer_chunks_empty() {
        let mut storage = setup_storage();
        let embedder = FallbackEmbedder::new(DEFAULT_DIMENSIONS);

        // Create buffer with no chunks
        let buffer = Buffer::from_named("empty.txt".to_string(), String::new());
        let buffer_id = storage.add_buffer(&buffer).unwrap();

        let count = embed_buffer_chunks(&mut storage, &embedder, buffer_id).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_buffer_fully_embedded_empty() {
        let mut storage = setup_storage();

        // Create buffer with no chunks
        let buffer = Buffer::from_named("empty.txt".to_string(), String::new());
        let buffer_id = storage.add_buffer(&buffer).unwrap();

        // Empty buffer should be "fully embedded"
        let result = buffer_fully_embedded(&storage, buffer_id).unwrap();
        assert!(result);
    }

    #[test]
    fn test_buffer_fully_embedded_with_embeddings() {
        let mut storage = setup_storage_with_chunks();
        let embedder = FallbackEmbedder::new(DEFAULT_DIMENSIONS);

        // Before embedding
        let result = buffer_fully_embedded(&storage, 1).unwrap();
        assert!(!result);

        // Embed all chunks
        embed_buffer_chunks(&mut storage, &embedder, 1).unwrap();

        // After embedding
        let result = buffer_fully_embedded(&storage, 1).unwrap();
        assert!(result);
    }

    #[test]
    fn test_hybrid_search_bm25_only() {
        let storage = setup_storage_with_chunks();
        let embedder = FallbackEmbedder::new(DEFAULT_DIMENSIONS);

        let config = SearchConfig::new().with_semantic(false).with_bm25(true);

        let results = hybrid_search(&storage, &embedder, "programming", &config).unwrap();
        // Should find "Rust is a systems programming language"
        assert!(!results.is_empty());
        assert!(results[0].bm25_score.is_some());
        assert!(results[0].semantic_score.is_none());
    }

    #[test]
    fn test_hybrid_search_semantic_only() {
        let mut storage = setup_storage_with_chunks();
        let embedder = FallbackEmbedder::new(DEFAULT_DIMENSIONS);

        // First embed the chunks
        embed_buffer_chunks(&mut storage, &embedder, 1).unwrap();

        let config = SearchConfig::new()
            .with_semantic(true)
            .with_bm25(false)
            .with_threshold(0.0); // Low threshold for fallback embedder

        let results = hybrid_search(&storage, &embedder, "programming language", &config).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].semantic_score.is_some());
        assert!(results[0].bm25_score.is_none());
    }

    #[test]
    fn test_hybrid_search_both() {
        let mut storage = setup_storage_with_chunks();
        let embedder = FallbackEmbedder::new(DEFAULT_DIMENSIONS);

        // First embed the chunks
        embed_buffer_chunks(&mut storage, &embedder, 1).unwrap();

        let config = SearchConfig::new()
            .with_semantic(true)
            .with_bm25(true)
            .with_threshold(0.0); // Low threshold for fallback embedder

        let results = hybrid_search(&storage, &embedder, "programming", &config).unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_semantic() {
        let mut storage = setup_storage_with_chunks();
        let embedder = FallbackEmbedder::new(DEFAULT_DIMENSIONS);

        // First embed the chunks
        embed_buffer_chunks(&mut storage, &embedder, 1).unwrap();

        let results = search_semantic(&storage, &embedder, "test query", 10, 0.0).unwrap();
        // Should return results with semantic scores only
        for result in &results {
            assert!(result.semantic_score.is_some());
            assert!(result.bm25_score.is_none());
        }
    }

    #[test]
    fn test_search_semantic_empty_embeddings() {
        let storage = setup_storage_with_chunks();
        let embedder = FallbackEmbedder::new(DEFAULT_DIMENSIONS);

        // Don't embed chunks - search should return empty
        let results = search_semantic(&storage, &embedder, "test query", 10, 0.5).unwrap();
        assert!(results.is_empty());
    }
}
