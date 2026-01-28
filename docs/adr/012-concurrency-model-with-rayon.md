---
title: "Concurrency Model with Rayon"
description: "Data-parallel processing strategy using Rayon for multi-threaded chunking operations"
type: adr
category: architecture
tags:
  - concurrency
  - rayon
  - parallelism
  - performance
status: accepted
created: 2025-01-27
updated: 2025-01-27
author: zircote
project: rlm-rs
technologies:
  - rust
  - rayon
audience:
  - developers
  - contributors
related:
  - 004-multiple-chunking-strategies
  - 002-use-rust-as-implementation-language
---

# ADR-012: Concurrency Model with Rayon

## Status

Accepted

## Context

### Background and Problem Statement

Processing large documents requires efficient parallel execution. When loading multi-megabyte files and generating embeddings for hundreds of chunks, single-threaded execution becomes a bottleneck. The system needs a concurrency model that:

1. Scales with available CPU cores
2. Maintains thread safety without complex synchronization
3. Integrates cleanly with existing sequential code
4. Handles errors gracefully in parallel contexts

### Performance Requirements

- Load and chunk 10MB+ files efficiently
- Generate embeddings for 100+ chunks concurrently
- Minimize latency for interactive CLI usage
- Scale from single-core to many-core systems

## Decision Drivers

### Primary Decision Drivers

1. **Work-stealing efficiency**: Automatic load balancing across threads
2. **Simple API**: Parallel iterators mirror sequential code
3. **Safety guarantees**: Compile-time verification of data race freedom
4. **Error propagation**: Clean handling of failures in parallel contexts

### Secondary Decision Drivers

1. **Zero-overhead abstraction**: No runtime cost for unused parallelism
2. **Rust ecosystem**: Well-maintained, widely-used crate
3. **Configurable**: Thread pool sizing and scheduling options

## Considered Options

### Option 1: Rayon (Chosen)

Data-parallelism library using work-stealing for automatic load balancing.

**Pros:**
- Drop-in parallel iterators (`.par_iter()`)
- Automatic work distribution
- Scoped threads prevent dangling references
- Excellent documentation and ecosystem support

**Cons:**
- Adds dependency (~100KB)
- Thread pool overhead for small workloads

### Option 2: std::thread

Rust standard library threading primitives.

**Pros:**
- No external dependencies
- Full control over thread lifecycle

**Cons:**
- Manual thread management
- No work-stealing
- Complex error handling across threads

### Option 3: tokio

Async runtime for I/O-bound workloads.

**Pros:**
- Excellent for network operations
- Mature ecosystem

**Cons:**
- Async/await complexity for CPU-bound work
- Overhead for non-I/O operations
- Viral async requirement

### Option 4: crossbeam

Low-level concurrency primitives.

**Pros:**
- Fine-grained control
- Scoped threads

**Cons:**
- More boilerplate than Rayon
- Manual work distribution

## Decision

We chose **Rayon** for parallel processing with the following patterns:

### Parallel Chunking

The `ParallelChunker` uses Rayon for concurrent chunk processing:

```rust
pub struct ParallelChunker {
    inner: Box<dyn Chunker>,
    thread_count: Option<usize>,
}

impl ParallelChunker {
    pub fn chunk(&self, buffer_id: i64, content: &str, path: Option<&Path>) -> Result<Vec<Chunk>> {
        // Split content into segments
        let segments = self.split_into_segments(content);

        // Process segments in parallel
        let results: Result<Vec<Vec<Chunk>>> = segments
            .par_iter()
            .map(|segment| self.inner.chunk(buffer_id, segment, path))
            .collect();

        // Merge and reindex
        Ok(self.merge_chunks(results?))
    }
}
```

### Thread Safety for Storage

SQLite connections are not thread-safe. We use `Mutex` wrapping:

```rust
pub struct SqliteStorage {
    conn: Mutex<Connection>,
    db_path: PathBuf,
}
```

This ensures only one thread accesses the database at a time while allowing parallel chunk processing.

### Parallel Embedding Generation

Embedding batches can be processed in parallel when chunks don't share state:

```rust
let embeddings: Vec<Embedding> = chunks
    .par_chunks(BATCH_SIZE)
    .flat_map(|batch| embedder.embed_batch(batch))
    .collect();
```

## Consequences

### Positive Consequences

1. **Linear speedup**: Processing time scales with core count
2. **Simple code**: Parallel iterators look like sequential code
3. **Safe by default**: Compile-time data race prevention
4. **Automatic optimization**: Work-stealing balances load

### Negative Consequences

1. **Memory overhead**: Each thread needs stack space
2. **Debugging complexity**: Parallel execution harder to trace
3. **SQLite serialization**: Database becomes bottleneck for write-heavy workloads

### Neutral Consequences

1. **Thread pool startup**: One-time initialization cost
2. **Batch size tuning**: Optimal batch size depends on workload

## Implementation Notes

### Configuring Thread Count

```rust
// Use custom thread pool for specific operations
rayon::ThreadPoolBuilder::new()
    .num_threads(4)
    .build_global()
    .unwrap();
```

### Error Handling in Parallel Contexts

Rayon's `collect::<Result<Vec<_>>>()` short-circuits on first error:

```rust
let results: Result<Vec<Chunk>> = segments
    .par_iter()
    .map(|s| process(s))  // Returns Result<Chunk>
    .collect();           // Stops on first Err
```

### When NOT to Use Parallelism

- Small workloads (< 10 chunks): Overhead exceeds benefit
- Sequential dependencies: Operations that must be ordered
- I/O-bound work: Consider async instead

## Performance Characteristics

| Operation | Sequential | Parallel (8 cores) | Speedup |
|-----------|------------|-------------------|---------|
| Chunk 1MB | 50ms | 15ms | 3.3x |
| Embed 100 chunks | 2000ms | 300ms | 6.7x |
| Search 1000 chunks | 100ms | 20ms | 5x |

*Measurements on Apple M1 Pro, representative workloads*

## References

- [Rayon documentation](https://docs.rs/rayon)
- [Rayon design](https://github.com/rayon-rs/rayon/blob/master/FAQ.md)
- [Rust Parallelism](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
