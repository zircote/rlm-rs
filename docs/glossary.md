# Glossary

A comprehensive reference for terminology used in rlm-rs and the RLM ecosystem.

## Core Concepts

### Buffer

A text container in the RLM system, typically loaded from a file or direct input. Buffers can be named, chunked, and searched.

**Example**: Loading `README.md` creates a buffer that can be referenced by name or ID.

### Chunk

A segment of text created by splitting a buffer according to a chunking strategy. Each chunk has a unique ID and can be embedded for semantic search.

**Example**: A markdown file might be split into chunks at each heading boundary.

### Chunking Strategy

An algorithm for splitting text into chunks. RLM-RS supports:
- **Semantic**: Natural language boundaries (headings, paragraphs)
- **Code**: Language-aware boundaries (functions, classes)
- **Fixed**: Fixed-size chunks with optional overlap
- **Parallel**: Multi-threaded fixed chunking for large files

### Embedding

A high-dimensional vector representation of text that captures semantic meaning. RLM-RS uses the BGE-M3 model to generate 1024-dimensional embeddings.

**Purpose**: Enables semantic search by finding chunks with similar meaning, not just matching keywords.

### Pass-by-Reference

An architectural pattern where chunk IDs are shared instead of copying full text content. This reduces token usage and improves efficiency.

**Example**: Instead of copying 10KB of text, share "chunk ID 42" and retrieve it on demand.

## Search Concepts

### BM25 (Best Match 25)

A ranking function used for keyword-based search. BM25 scores documents based on term frequency and document length.

**Use Case**: Finding chunks that contain specific keywords or phrases.

### Semantic Search

Search based on meaning rather than exact keyword matches. Uses embeddings and vector similarity.

**Example**: Searching for "installation" might return chunks about "setup" or "getting started".

### Hybrid Search

Combines semantic search and BM25 keyword search using Reciprocal Rank Fusion (RRF).

**Advantage**: Gets both semantic understanding and keyword precision.

### RRF (Reciprocal Rank Fusion)

An algorithm for combining multiple ranked lists into a single ranking. RLM-RS uses RRF to merge semantic and BM25 search results.

**Formula**: `score(chunk) = Σ 1 / (k + rank(chunk))` where k is a constant (default 60).

### Cosine Similarity

A measure of similarity between two vectors, ranging from -1 to 1. Higher values indicate more similar embeddings.

**Use**: Ranking chunks by semantic similarity to a query.

### Top-K

The number of top-ranked results to return from a search query.

**Example**: `--top-k 5` returns the 5 most relevant chunks.

## Technical Terms

### HNSW (Hierarchical Navigable Small World)

A graph-based algorithm for approximate nearest neighbor search in high-dimensional spaces. Optional feature in RLM-RS for faster semantic search.

**Trade-off**: Faster search at the cost of some accuracy.

### SQLite

An embedded relational database used by RLM-RS for persistent state storage.

**Location**: `.rlm/rlm-state.db` in your working directory.

### Memory-Mapped I/O (mmap)

A technique for efficiently reading large files by mapping them directly into memory.

**Benefit**: Reduces memory usage and improves performance for large files.

### Unicode Grapheme Cluster

The user-perceived character unit in Unicode, which may consist of multiple codepoints.

**Example**: The emoji "👨‍👩‍👧‍👦" is a single grapheme cluster made of multiple codepoints.

**Importance**: RLM-RS chunks at grapheme boundaries to avoid splitting multi-codepoint characters.

## RLM Architecture

### Root LLM

The primary AI model orchestrating the overall task. In Claude Code integration, this is the main conversation using Opus or Sonnet.

**Role**: Decomposes complex tasks and manages sub-LLM calls.

### Sub-LLM

Smaller, faster AI models used for analyzing individual chunks. Typically Haiku in Claude Code integration.

**Role**: Processes specific chunks and returns findings to the root LLM.

### External Environment

The persistent storage and tools used by the RLM system. For rlm-rs, this includes the SQLite database and CLI commands.

**Components**:
- SQLite database (`.rlm/rlm-state.db`)
- CLI commands (`load`, `search`, `chunk get`, etc.)
- Embedding models (BGE-M3)
- Vector indices (optional HNSW)

### Dispatch

The process of splitting chunks into batches for parallel processing by sub-LLMs.

**Command**: `rlm-rs dispatch --buffer docs --batch-size 5`

### Aggregate

The process of combining findings from multiple sub-LLM analyses into a coherent summary.

**Command**: `rlm-rs aggregate`

## Feature Flags

### fastembed-embeddings

Enables semantic embeddings using the BGE-M3 ONNX model.

**Default**: Enabled

**Build**: `cargo build --features fastembed-embeddings`

### usearch-hnsw

Enables HNSW approximate nearest neighbor search for faster semantic search.

**Default**: Disabled

**Build**: `cargo build --features usearch-hnsw`

### full-search

Enables both FastEmbed embeddings and USearch HNSW.

**Build**: `cargo build --features full-search`

## CLI Terms

### Buffer Name

An optional human-readable identifier for a buffer. If not provided, a timestamp-based name is generated.

**Example**: `--name readme` creates a buffer named "readme".

### Chunk ID

A unique integer identifier for a chunk, assigned by the storage layer.

**Usage**: `rlm-rs chunk get 42` retrieves chunk with ID 42.

### Content Type

The file extension or MIME type associated with a buffer.

**Auto-detected**: From file extension when loading from file.

### Context Variable

A key-value pair stored in the RLM state for sharing data between commands.

**Example**: `rlm-rs var set task "analyze performance"`

### Global Variable

A persistent variable stored in the database that survives across sessions.

**Example**: `rlm-rs global set model "claude-opus"`

## Common Abbreviations

- **ADR**: Architecture Decision Record
- **API**: Application Programming Interface
- **BGE**: Beijing Academy of Artificial Intelligence General Embedding
- **BM25**: Best Match 25 (ranking function)
- **CLI**: Command-Line Interface
- **HNSW**: Hierarchical Navigable Small World (graph algorithm)
- **LLM**: Large Language Model
- **MSRV**: Minimum Supported Rust Version
- **ONNX**: Open Neural Network Exchange
- **RLM**: Recursive Language Model
- **RRF**: Reciprocal Rank Fusion
- **SQL**: Structured Query Language

## Related Terms

### Context Window

The maximum amount of text an LLM can process in a single request, measured in tokens.

**Claude Models**:
- Opus: 200K tokens
- Sonnet: 200K tokens
- Haiku: 200K tokens

**RLM Purpose**: Process content larger than the context window via chunking.

### Token

A unit of text processed by an LLM. Roughly 4 characters per token for English.

**Optimization**: RLM-RS reduces token usage via pass-by-reference.

### Recursive Processing

A pattern where an LLM delegates subtasks to other LLM instances, creating a hierarchy of processing.

**RLM Implementation**: Root LLM delegates chunk analysis to sub-LLMs.

## See Also

- **[Architecture](architecture.md)** - System design and components
- **[RLM-Inspired Design](rlm-inspired-design.md)** - Connection to RLM paper
- **[CLI Reference](cli-reference.md)** - Command documentation
- **[API Reference](api.md)** - Rust library documentation

---

**Last Updated**: 2026-02-18  
**Version**: 1.2.4
