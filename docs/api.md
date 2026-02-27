# RLM-RS API Reference

Rust library API documentation for `rlm-cli`.

## Overview

`rlm-cli` can be used as both a CLI tool and a Rust library. This document covers the library API for programmatic integration.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
rlm-cli = "1.2"
```

Basic usage:

```rust
use rlm_rs::{Buffer, Chunker, SemanticChunker, SqliteStorage, Storage};

fn main() -> rlm_rs::Result<()> {
    // Initialize storage
    let mut storage = SqliteStorage::open(".rlm/rlm-state.db")?;
    storage.init()?;

    // Create a buffer from content
    let buffer = Buffer::from_content("Hello, world!".to_string());
    let buffer_id = storage.add_buffer(&buffer)?;

    // Chunk the content
    let chunker = SemanticChunker::new();
    let chunks = chunker.chunk(buffer_id, &buffer.content, None)?;

    // Store chunks
    storage.add_chunks(buffer_id, &chunks)?;

    Ok(())
}
```

---

## Core Types

### `Buffer`

Represents a text buffer loaded into the RLM system.

**Location:** `rlm_rs::core::Buffer`

```rust
pub struct Buffer {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub source: Option<PathBuf>,
    pub content: String,
    pub metadata: BufferMetadata,
}
```

#### Constructors

| Method | Description |
|--------|-------------|
| `Buffer::from_content(content: String)` | Create from string content |
| `Buffer::from_file(path: PathBuf, content: String)` | Create from file path and content |
| `Buffer::from_named(name: String, content: String)` | Create with explicit name |

#### Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `size()` | `usize` | Buffer size in bytes |
| `line_count()` | `usize` | Number of lines (cached) |
| `slice(start, end)` | `Option<&str>` | Get content slice |
| `peek(len)` | `&str` | Preview first N bytes |
| `peek_end(len)` | `&str` | Preview last N bytes |
| `is_empty()` | `bool` | Check if empty |
| `display_name()` | `String` | Human-readable name |
| `compute_hash()` | `()` | Compute content hash |

#### Example

```rust
use rlm_rs::Buffer;
use std::path::PathBuf;

// From content
let buffer = Buffer::from_content("Hello, world!".to_string());
assert_eq!(buffer.size(), 13);

// From file
let buffer = Buffer::from_file(
    PathBuf::from("document.md"),
    std::fs::read_to_string("document.md")?,
);
assert!(buffer.source.is_some());

// Slicing
if let Some(slice) = buffer.slice(0, 100) {
    println!("First 100 bytes: {}", slice);
}
```

---

### `BufferMetadata`

Metadata associated with a buffer.

```rust
pub struct BufferMetadata {
    pub content_type: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub size: usize,
    pub line_count: Option<usize>,
    pub chunk_count: Option<usize>,
    pub content_hash: Option<String>,
}
```

---

### `Chunk`

Represents a segment of buffer content.

**Location:** `rlm_rs::core::Chunk`

```rust
pub struct Chunk {
    pub id: Option<i64>,
    pub buffer_id: i64,
    pub content: String,
    pub byte_range: Range<usize>,
    pub index: usize,
    pub metadata: ChunkMetadata,
}
```

#### Constructors

| Method | Description |
|--------|-------------|
| `Chunk::new(buffer_id, content, byte_range, index)` | Create new chunk |
| `Chunk::with_strategy(buffer_id, content, byte_range, index, strategy)` | Create with strategy name |
| `ChunkBuilder::new()` | Fluent builder pattern |

#### Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `size()` | `usize` | Chunk content size in bytes |
| `range_size()` | `usize` | Byte range size |
| `start()` | `usize` | Start byte offset |
| `end()` | `usize` | End byte offset |
| `is_empty()` | `bool` | Check if empty |
| `estimate_tokens()` | `usize` | Estimate token count (~4 chars/token) |
| `estimate_tokens_accurate()` | `usize` | Accurate token estimate (word-aware) |
| `preview(max_len)` | `&str` | Preview first N characters |
| `overlaps_with(range)` | `bool` | Check if overlaps with range |
| `contains_offset(offset)` | `bool` | Check if contains byte offset |
| `compute_hash()` | `()` | Compute content hash |

#### Example

```rust
use rlm_rs::Chunk;

let chunk = Chunk::new(
    1,                          // buffer_id
    "Hello, world!".to_string(), // content
    0..13,                       // byte_range
    0,                           // index
);

assert_eq!(chunk.size(), 13);
assert_eq!(chunk.estimate_tokens(), 4); // ~4 chars per token
assert!(chunk.contains_offset(5));
```

#### Builder Pattern

```rust
use rlm_rs::core::chunk::ChunkBuilder;

let chunk = ChunkBuilder::new()
    .buffer_id(1)
    .content("Hello, world!".to_string())
    .byte_range(0..13)
    .index(0)
    .strategy("semantic")
    .has_overlap(false)
    .build();
```

---

### `ChunkMetadata`

Metadata associated with a chunk.

```rust
pub struct ChunkMetadata {
    pub strategy: Option<String>,
    pub token_count: Option<usize>,
    pub line_range: Option<Range<usize>>,
    pub created_at: i64,
    pub content_hash: Option<String>,
    pub has_overlap: bool,
    pub custom: Option<String>,
}
```

---

### `Context`

Manages variables and state.

**Location:** `rlm_rs::core::Context`

```rust
pub struct Context {
    pub variables: HashMap<String, ContextValue>,
    pub globals: HashMap<String, ContextValue>,
    pub buffer_ids: Vec<i64>,
    pub cwd: Option<String>,
    pub metadata: ContextMetadata,
}
```

---

### `ContextValue`

Typed values for context variables.

```rust
pub enum ContextValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    List(Vec<ContextValue>),
    Map(HashMap<String, ContextValue>),
    Null,
}
```

---

## Chunking

### `Chunker` Trait

All chunking strategies implement this trait.

**Location:** `rlm_rs::chunking::Chunker`

```rust
pub trait Chunker: Send + Sync {
    fn chunk(
        &self,
        buffer_id: i64,
        text: &str,
        metadata: Option<&ChunkMetadata>,
    ) -> Result<Vec<Chunk>>;

    fn name(&self) -> &'static str;
    fn supports_parallel(&self) -> bool;
    fn description(&self) -> &'static str;
    fn validate(&self, metadata: Option<&ChunkMetadata>) -> Result<()>;
}
```

### Chunking Strategies

#### `SemanticChunker`

Unicode-aware chunking that respects sentence and paragraph boundaries.

```rust
use rlm_rs::chunking::{Chunker, SemanticChunker};

let chunker = SemanticChunker::new();
// Or with custom size:
let chunker = SemanticChunker::with_size(20_000);
// Or with size and overlap:
let chunker = SemanticChunker::with_size_and_overlap(20_000, 500);

let chunks = chunker.chunk(1, "Your long text...", None)?;
```

**Best for:** Markdown, prose, code, structured documents.

---

#### `FixedChunker`

Simple character-based chunking at exact boundaries.

```rust
use rlm_rs::chunking::{Chunker, FixedChunker};

let chunker = FixedChunker::new();
// Or with custom size:
let chunker = FixedChunker::with_size(50_000);
// Or with size and overlap:
let chunker = FixedChunker::with_size_and_overlap(50_000, 1000);

let chunks = chunker.chunk(1, "Your long text...", None)?;
```

**Best for:** Logs, plain text, binary-safe content.

---

#### `ParallelChunker`

Multi-threaded chunking using Rayon for large files.

```rust
use rlm_rs::chunking::{Chunker, ParallelChunker, SemanticChunker};

let inner = SemanticChunker::new();
let chunker = ParallelChunker::new(inner);

let chunks = chunker.chunk(1, "Your very large text...", None)?;
```

**Best for:** Large files (>10MB).

---

#### `CodeChunker`

Language-aware chunking at function and class boundaries.

```rust
use rlm_rs::chunking::{Chunker, CodeChunker, ChunkerMetadata};

let chunker = CodeChunker::new();

// Specify content type for language detection
let metadata = ChunkerMetadata::new().content_type("rs");
let chunks = chunker.chunk(1, rust_code, Some(&metadata))?;
```

**Supported Languages:**
- Rust (.rs) - `fn`, `impl`, `struct`, `enum`, `mod`
- Python (.py) - `def`, `class`, `async def`
- JavaScript/TypeScript (.js, .jsx, .ts, .tsx) - `function`, `class`
- Go (.go) - `func`, `type`
- Java (.java) - `class`, `interface`, methods
- C/C++ (.c, .cpp, .h, .hpp) - functions
- Ruby (.rb) - `def`, `class`, `module`
- PHP (.php) - `function`, `class`

**Best for:** Source code files where semantic boundaries matter.

---

### Factory Functions

```rust
use rlm_rs::chunking::{create_chunker, available_strategies};

// Create chunker by name
let chunker = create_chunker("semantic")?;
let chunker = create_chunker("code")?;  // Language-aware chunking
let chunker = create_chunker("fixed")?;
let chunker = create_chunker("parallel")?;

// List available strategies
let strategies = available_strategies(); // ["fixed", "semantic", "code", "parallel"]
```

---

### Chunking Configuration

```rust
use rlm_rs::chunking::traits::ChunkMetadata;

let metadata = ChunkMetadata::new()
    .with_size_and_overlap(30_000, 500)
    .source("document.md")
    .content_type("md")
    .preserve_sentences(true)
    .max_chunks(100);

let chunks = chunker.chunk(1, text, Some(&metadata))?;
```

---

### Constants

```rust
use rlm_rs::chunking::{DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP, MAX_CHUNK_SIZE};

// DEFAULT_CHUNK_SIZE = 3_000 (~750 tokens)
// DEFAULT_OVERLAP = 500
// MAX_CHUNK_SIZE = 50_000
```

---

## Storage

### `Storage` Trait

Interface for persistent storage backends.

**Location:** `rlm_rs::storage::Storage`

```rust
pub trait Storage: Send + Sync {
    // Lifecycle
    fn init(&mut self) -> Result<()>;
    fn is_initialized(&self) -> Result<bool>;
    fn reset(&mut self) -> Result<()>;

    // Context
    fn save_context(&mut self, context: &Context) -> Result<()>;
    fn load_context(&self) -> Result<Option<Context>>;
    fn delete_context(&mut self) -> Result<()>;

    // Buffers
    fn add_buffer(&mut self, buffer: &Buffer) -> Result<i64>;
    fn get_buffer(&self, id: i64) -> Result<Option<Buffer>>;
    fn get_buffer_by_name(&self, name: &str) -> Result<Option<Buffer>>;
    fn list_buffers(&self) -> Result<Vec<Buffer>>;
    fn update_buffer(&mut self, buffer: &Buffer) -> Result<()>;
    fn delete_buffer(&mut self, id: i64) -> Result<()>;
    fn buffer_count(&self) -> Result<usize>;

    // Chunks
    fn add_chunks(&mut self, buffer_id: i64, chunks: &[Chunk]) -> Result<()>;
    fn get_chunks(&self, buffer_id: i64) -> Result<Vec<Chunk>>;
    fn get_chunk(&self, id: i64) -> Result<Option<Chunk>>;
    fn delete_chunks(&mut self, buffer_id: i64) -> Result<()>;
    fn chunk_count(&self, buffer_id: i64) -> Result<usize>;

    // Utilities
    fn export_buffers(&self) -> Result<String>;
    fn stats(&self) -> Result<StorageStats>;
}
```

---

### `SqliteStorage`

SQLite-backed storage implementation.

**Location:** `rlm_rs::storage::SqliteStorage`

```rust
use rlm_rs::{SqliteStorage, Storage};

// Open or create database
let mut storage = SqliteStorage::open(".rlm/rlm-state.db")?;

// Initialize schema
storage.init()?;

// Check if initialized
if storage.is_initialized()? {
    println!("Database ready");
}

// Get statistics
let stats = storage.stats()?;
println!("Buffers: {}", stats.buffer_count);
println!("Chunks: {}", stats.chunk_count);
```

---

### `StorageStats`

Storage statistics.

```rust
pub struct StorageStats {
    pub buffer_count: usize,
    pub chunk_count: usize,
    pub total_content_size: usize,
    pub has_context: bool,
    pub schema_version: u32,
    pub db_size: Option<u64>,
}
```

---

### Embedding and Search

Enable full search capabilities with features:

```toml
[dependencies]
# Default: includes fastembed embeddings
rlm-cli = "1.2"

# Full search with HNSW index
rlm-cli = { version = "1.2", features = ["full-search"] }
```

#### Generating Embeddings

```rust
use rlm_rs::search::{embed_buffer_chunks, embed_buffer_chunks_incremental};
use rlm_rs::embedding::create_embedder;

// Create embedder (BGE-M3 or fallback)
let embedder = create_embedder()?;

// Embed all chunks in a buffer
let count = embed_buffer_chunks(&mut storage, embedder.as_ref(), buffer_id)?;

// Incremental embedding (only new/changed chunks)
let result = embed_buffer_chunks_incremental(
    &mut storage,
    embedder.as_ref(),
    buffer_id,
    false,  // force_reembed
)?;
println!("Embedded: {}, Skipped: {}", result.embedded_count, result.skipped_count);
```

#### Hybrid Search

```rust
use rlm_rs::search::{hybrid_search, SearchConfig};

// Builder pattern (recommended)
let config = SearchConfig::new()
    .with_top_k(10)
    .with_threshold(0.3)
    .with_rrf_k(60)
    .with_semantic(true)
    .with_bm25(true);

let results = hybrid_search(&storage, embedder.as_ref(), "your query", &config)?;
for result in results {
    println!("Chunk {}: score {:.4}", result.chunk_id, result.score);
}
```

#### `SearchConfig` Builder

**Location:** `rlm_rs::search::SearchConfig`

`SearchConfig` exposes a builder API. All methods consume `self` and return a new `SearchConfig`.

| Method | Description | Default |
|--------|-------------|---------|
| `SearchConfig::new()` | Create config with defaults | — |
| `.with_top_k(n)` | Maximum results to return | `10` |
| `.with_threshold(f32)` | Minimum semantic similarity score | `0.3` |
| `.with_rrf_k(u32)` | RRF *k* smoothing parameter | `60` |
| `.with_semantic(bool)` | Enable/disable semantic search leg | `true` |
| `.with_bm25(bool)` | Enable/disable BM25 search leg | `true` |

**Constants:**

```rust
use rlm_rs::search::{DEFAULT_SIMILARITY_THRESHOLD, DEFAULT_TOP_K};
// DEFAULT_PREVIEW_LEN is available via rlm_rs::search::DEFAULT_PREVIEW_LEN (not re-exported at crate root)
// DEFAULT_SIMILARITY_THRESHOLD = 0.3
// DEFAULT_TOP_K = 10
// DEFAULT_PREVIEW_LEN = 150  (characters in content preview)
```

#### Standalone Search Functions

```rust
use rlm_rs::search::{search_semantic, search_bm25};

// Semantic-only search (cosine similarity on embeddings)
let semantic_results = search_semantic(&storage, embedder.as_ref(), "query", 10, 0.3)?;

// BM25-only full-text search (no embeddings required)
let bm25_results = search_bm25(&storage, "query", 10)?;
```

#### `SearchResult` Fields

```rust
pub struct SearchResult {
    pub chunk_id: i64,           // Database ID of the chunk
    pub buffer_id: i64,          // Buffer this chunk belongs to
    pub index: usize,            // 0-based position within the buffer
    pub score: f64,              // Combined RRF score (higher is better)
    pub semantic_score: Option<f32>,  // Cosine similarity (if semantic search ran)
    pub bm25_score: Option<f64>,      // BM25 relevance (if BM25 search ran)
    pub content_preview: Option<String>, // Snippet; populated by populate_previews()
}
```

#### `populate_previews`

Fetches chunk content and fills `content_preview` on each `SearchResult`.

```rust
use rlm_rs::search::{hybrid_search, populate_previews, DEFAULT_PREVIEW_LEN};

let mut results = hybrid_search(&storage, embedder.as_ref(), "query", &config)?;
populate_previews(&storage, &mut results, DEFAULT_PREVIEW_LEN)?;

for r in &results {
    println!("[{}] {}", r.chunk_id, r.content_preview.as_deref().unwrap_or(""));
}
```

#### Embedding Status Utilities

```rust
use rlm_rs::search::{buffer_fully_embedded, check_model_mismatch,
                     get_embedding_model_info, EmbeddingModelInfo};

// Check if every chunk in a buffer has an embedding
let ready = buffer_fully_embedded(&storage, buffer_id)?;

// Detect if existing embeddings used a different model
if let Some(old_model) = check_model_mismatch(&storage, buffer_id, "BGE-M3")? {
    eprintln!("Warning: buffer was embedded with '{old_model}', current model is BGE-M3");
}

// Full model breakdown per buffer
let info: EmbeddingModelInfo = get_embedding_model_info(&storage, buffer_id)?;
println!("Total embeddings: {}", info.total_embeddings);
println!("Mixed models: {}", info.has_mixed_models);
for (model, count) in &info.models {
    println!("  {:?}: {} embeddings", model, count);
}
```

`EmbeddingModelInfo` fields:

| Field | Type | Description |
|-------|------|-------------|
| `models` | `Vec<(Option<String>, i64)>` | Model name → embedding count pairs |
| `total_embeddings` | `i64` | Sum of all embeddings for the buffer |
| `has_mixed_models` | `bool` | `true` when more than one model is present |

#### Incremental Embedding

`embed_buffer_chunks_incremental` skips chunks that already have an up-to-date embedding, making repeated calls efficient for large or frequently-updated buffers.

```rust
use rlm_rs::search::embed_buffer_chunks_incremental;

let result = embed_buffer_chunks_incremental(
    &mut storage,
    embedder.as_ref(),
    buffer_id,
    false,  // force_reembed: set true to replace embeddings from a different model
)?;

println!("Embedded: {}", result.embedded_count);
println!("Skipped (already current): {}", result.skipped_count);
println!("Replaced (model changed): {}", result.replaced_count);
println!("Progress: {:.1}%", result.completion_percentage());
println!("Had changes: {}", result.had_changes());
```

`IncrementalEmbedResult` fields:

| Field | Type | Description |
|-------|------|-------------|
| `embedded_count` | `usize` | New embeddings created this run |
| `skipped_count` | `usize` | Chunks already embedded with current model |
| `replaced_count` | `usize` | Old embeddings replaced (different model) |
| `total_chunks` | `usize` | Total chunks in the buffer |
| `model_name` | `String` | Model used for this run |

Helper methods: `.had_changes() -> bool`, `.completion_percentage() -> f64`.

#### Reciprocal Rank Fusion (RRF)

**Location:** `rlm_rs::search::{RrfConfig, reciprocal_rank_fusion, weighted_rrf}`

RRF merges multiple independently-ranked lists into a single fused ranking without requiring score normalisation.

```rust
use rlm_rs::search::{RrfConfig, reciprocal_rank_fusion, weighted_rrf};

// Standard RRF — equal weight to every list
let config = RrfConfig::new(60);   // k=60 is the paper's recommended default
let semantic_ids = vec![3_i64, 1, 5, 2, 4];
let bm25_ids     = vec![1_i64, 3, 2, 5, 4];
let fused = reciprocal_rank_fusion(&[&semantic_ids, &bm25_ids], &config);

// Weighted RRF — give semantic results twice the weight of BM25
let weighted = weighted_rrf(
    &[(&semantic_ids, 2.0), (&bm25_ids, 1.0)],
    &config,
);

for (chunk_id, score) in &fused {
    println!("chunk {chunk_id}: {score:.4}");
}
```

`RrfConfig` fields:

| Field | Type | Description |
|-------|------|-------------|
| `k` | `u32` | Smoothing constant — higher values give more weight to lower-ranked items (default: `60`) |

#### HNSW Index (Optional)

When the `usearch-hnsw` feature is enabled:

```rust
#[cfg(feature = "usearch-hnsw")]
use rlm_rs::search::{HnswIndex, HnswConfig};

let config = HnswConfig::default();
let mut index = HnswIndex::new(1024, config)?;  // 1024 dimensions

// Add vectors
index.add(chunk_id, &embedding)?;

// Search
let results = index.search(&query_embedding, 10)?;
```

---

## I/O

### File Reading

**Location:** `rlm_rs::io`

```rust
use rlm_rs::io::{read_file, read_file_mmap};
use std::path::Path;

// Standard file read
let content = read_file(Path::new("document.md"))?;

// Memory-mapped read (efficient for large files)
let content = read_file_mmap(Path::new("large-file.txt"))?;
```

---

### File Writing

```rust
use rlm_rs::io::{write_file, write_chunks};
use std::path::Path;

// Write content to file
write_file(Path::new("output.txt"), "content")?;

// Write chunks to directory
write_chunks(
    Path::new(".rlm/chunks"),
    &chunks,
    "chunk",  // prefix
)?;
// Creates: chunk_0.txt, chunk_1.txt, ...
```

---

### Unicode Utilities

```rust
use rlm_rs::io::{find_char_boundary, validate_utf8};

// Find valid UTF-8 boundary at or before position
let boundary = find_char_boundary("Hello, 世界!", 8);

// Validate UTF-8
validate_utf8(bytes)?;
```

---

## Error Handling

### Error Types

**Location:** `rlm_rs::error`

```rust
use rlm_rs::{Error, Result};

pub enum Error {
    Storage(StorageError),
    Chunking(ChunkingError),
    Io(IoError),
    Command(CommandError),
    InvalidState { message: String },
    Config { message: String },
}
```

### `StorageError`

```rust
pub enum StorageError {
    Database(String),
    NotInitialized,
    ContextNotFound,
    BufferNotFound { identifier: String },
    ChunkNotFound { id: i64 },
    Migration(String),
    Transaction(String),
    Serialization(String),
}
```

### `ChunkingError`

```rust
pub enum ChunkingError {
    InvalidUtf8 { offset: usize },
    ChunkTooLarge { size: usize, max: usize },
    InvalidConfig { reason: String },
    OverlapTooLarge { overlap: usize, size: usize },
    ParallelFailed { reason: String },
    SemanticFailed(String),
    Regex(String),
    UnknownStrategy { name: String },
}
```

### `IoError`

```rust
pub enum IoError {
    FileNotFound { path: String },
    ReadFailed { path: String, reason: String },
    WriteFailed { path: String, reason: String },
    MmapFailed { path: String, reason: String },
    DirectoryFailed { path: String, reason: String },
    PathTraversal { path: String },
    Generic(String),
}
```

---

## CLI Integration

### Using the CLI Types

```rust
use rlm_rs::cli::{Cli, Commands, execute};
use clap::Parser;

// Parse arguments
let cli = Cli::parse();

// Execute command
let output = execute(&cli)?;
println!("{}", output);
```

### Output Formats

```rust
use rlm_rs::cli::OutputFormat;

match format {
    OutputFormat::Text => println!("{}", result),
    OutputFormat::Json => println!("{}", serde_json::to_string(&result)?),
}
```

---

## Re-exports

The crate root re-exports commonly used types:

```rust
// Error handling
pub use error::{Error, Result};

// Core types
pub use core::{Buffer, BufferMetadata, Chunk, ChunkMetadata, Context, ContextValue};

// Storage
pub use storage::{DEFAULT_DB_PATH, SqliteStorage, Storage};

// Chunking
pub use chunking::{Chunker, FixedChunker, SemanticChunker, available_strategies, create_chunker};

// CLI
pub use cli::{Cli, Commands, OutputFormat};

// Embedding
#[cfg(feature = "fastembed-embeddings")]
pub use embedding::FastEmbedEmbedder;
pub use embedding::{DEFAULT_DIMENSIONS, Embedder, FallbackEmbedder, cosine_similarity, create_embedder};

// Search
pub use search::{
    DEFAULT_SIMILARITY_THRESHOLD, DEFAULT_TOP_K,
    RrfConfig, SearchConfig, SearchResult,
    buffer_fully_embedded, embed_buffer_chunks, hybrid_search,
    reciprocal_rank_fusion, search_bm25, search_semantic, weighted_rrf,
};
```

---

## Complete Example

```rust
use rlm_rs::{
    Buffer, Chunk, Chunker, SemanticChunker, SqliteStorage, Storage, Result,
};
use std::path::PathBuf;

fn process_document(path: &str) -> Result<()> {
    // 1. Initialize storage
    let mut storage = SqliteStorage::open(".rlm/rlm-state.db")?;
    storage.init()?;

    // 2. Read file content
    let content = std::fs::read_to_string(path)?;

    // 3. Create buffer
    let buffer = Buffer::from_file(PathBuf::from(path), content.clone());
    let buffer_id = storage.add_buffer(&buffer)?;

    // 4. Chunk the content
    let chunker = SemanticChunker::with_size_and_overlap(3_000, 500);
    let chunks = chunker.chunk(buffer_id, &content, None)?;

    println!("Created {} chunks", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        println!(
            "  Chunk {}: {} bytes, ~{} tokens",
            i,
            chunk.size(),
            chunk.estimate_tokens()
        );
    }

    // 5. Store chunks
    storage.add_chunks(buffer_id, &chunks)?;

    // 6. Query stored data
    let stats = storage.stats()?;
    println!("\nStorage stats:");
    println!("  Buffers: {}", stats.buffer_count);
    println!("  Chunks: {}", stats.chunk_count);
    println!("  Total size: {} bytes", stats.total_content_size);

    Ok(())
}
```

---

## See Also

- [README.md](../README.md) - Project overview
- [Architecture](architecture.md) - Internal architecture
- [CLI Reference](cli-reference.md) - Command-line interface
- [docs.rs/rlm-cli](https://docs.rs/rlm-cli) - Auto-generated rustdoc
