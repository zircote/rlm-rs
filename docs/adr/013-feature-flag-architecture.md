---
title: "Feature Flag Architecture"
description: "Cargo feature flags for optional functionality and binary size optimization"
type: adr
category: architecture
tags:
  - features
  - cargo
  - optional-dependencies
  - binary-size
status: accepted
created: 2025-01-27
updated: 2025-01-27
author: zircote
project: rlm-rs
technologies:
  - rust
  - cargo
audience:
  - developers
  - packagers
related:
  - 007-embedded-embedding-model
  - 010-switch-to-bge-m3-model
---

# ADR-013: Feature Flag Architecture

## Status

Accepted

## Context

### Background and Problem Statement

RLM-RS includes optional heavyweight dependencies that significantly impact binary size and compile time:

1. **fastembed-embeddings**: ONNX runtime + BGE-M3 model (~150MB binary impact)
2. **usearch-hnsw**: HNSW index for vector similarity (~5MB binary impact)
3. **full-search**: All search capabilities combined

Not all use cases require all features. A documentation analysis tool may not need embeddings. A simple grep-based workflow may not need vector search. Users should be able to opt-in to only the features they need.

### Size and Performance Considerations

| Feature | Binary Size Impact | Compile Time Impact |
|---------|-------------------|---------------------|
| Base (no features) | ~5MB | ~30s |
| fastembed-embeddings | +150MB | +5min |
| usearch-hnsw | +5MB | +30s |
| full-search | +155MB | +6min |

## Decision Drivers

### Primary Decision Drivers

1. **Binary size**: Enable minimal installations for constrained environments
2. **Compile time**: Speed up development builds by excluding unused features
3. **Flexibility**: Different deployment scenarios need different capabilities

### Secondary Decision Drivers

1. **Default experience**: Common use cases should work out of the box
2. **Graceful degradation**: Missing features should fail clearly
3. **Documentation**: Feature requirements should be obvious

## Considered Options

### Option 1: Cargo Feature Flags (Chosen)

Standard Rust mechanism for conditional compilation.

**Pros:**
- Standard, well-understood pattern
- Compile-time feature selection
- Zero runtime cost for disabled features
- Enables optional dependencies

**Cons:**
- Feature combinations can be complex
- Must test all flag combinations in CI

### Option 2: Runtime Configuration

Load capabilities at runtime based on config.

**Pros:**
- Single binary for all scenarios
- Dynamic capability discovery

**Cons:**
- Larger binary always
- Runtime overhead for feature checks
- Complex error handling

### Option 3: Multiple Binaries

Separate binaries for different use cases.

**Pros:**
- Clear separation
- Optimized per use case

**Cons:**
- Distribution complexity
- Maintenance burden
- User confusion

## Decision

We use **Cargo feature flags** with the following architecture:

### Feature Definitions

```toml
[features]
default = ["fastembed-embeddings"]

# Embedding generation with BGE-M3 model via ONNX runtime
fastembed-embeddings = ["dep:fastembed"]

# HNSW index for fast approximate nearest neighbor search
usearch-hnsw = ["dep:usearch"]

# All search capabilities (semantic + vector index)
full-search = ["fastembed-embeddings", "usearch-hnsw"]
```

### Feature Matrix

| Feature | Semantic Search | BM25 Search | HNSW Index | Embedding Generation |
|---------|-----------------|-------------|------------|---------------------|
| (none) | No | Yes | No | No |
| fastembed-embeddings | Yes | Yes | No | Yes |
| usearch-hnsw | No | Yes | Yes | No |
| full-search | Yes | Yes | Yes | Yes |

### Default Feature

`fastembed-embeddings` is enabled by default because:
- Most users want semantic search capabilities
- BM25-only search is a specialized use case
- The RLM workflow relies on semantic chunking

### Conditional Compilation Patterns

```rust
// Optional embedding support
#[cfg(feature = "fastembed-embeddings")]
mod fastembed_impl;

#[cfg(feature = "fastembed-embeddings")]
pub use fastembed_impl::FastEmbedEmbedder;

// Fallback when embeddings disabled
#[cfg(not(feature = "fastembed-embeddings"))]
pub fn create_embedder() -> impl Embedder {
    FallbackEmbedder::new()  // Returns error on embed attempts
}
```

### Error Types for Missing Features

```rust
#[derive(Error, Debug)]
pub enum StorageError {
    #[cfg(feature = "fastembed-embeddings")]
    #[error("embedding error: {0}")]
    Embedding(#[from] EmbeddingError),

    #[cfg(feature = "usearch-hnsw")]
    #[error("vector search error: {0}")]
    VectorSearch(String),
}
```

## Consequences

### Positive Consequences

1. **Optimized binaries**: Users get only what they need
2. **Faster CI**: Can test core functionality without heavy deps
3. **Clear capabilities**: Feature flags document optional functionality
4. **Gradual adoption**: Start minimal, add features as needed

### Negative Consequences

1. **CI complexity**: Must test multiple feature combinations
2. **Documentation burden**: Must document feature requirements
3. **User confusion**: Feature selection adds decision point

### Neutral Consequences

1. **Conditional compilation**: Code has `#[cfg(...)]` annotations
2. **Feature propagation**: Dependencies may enable sub-features

## Implementation Notes

### Building with Specific Features

```bash
# Minimal build (BM25 search only)
cargo build --no-default-features

# Default build (with embeddings)
cargo build

# Full build (all features)
cargo build --all-features

# Specific combination
cargo build --features "fastembed-embeddings,usearch-hnsw"
```

### CI Testing Matrix

```yaml
jobs:
  test:
    strategy:
      matrix:
        features:
          - ""                        # No features
          - "fastembed-embeddings"    # Default
          - "usearch-hnsw"            # HNSW only
          - "full-search"             # All features
```

### Checking Feature at Runtime

For user-facing messages:

```rust
pub fn search_capabilities() -> Vec<&'static str> {
    let mut caps = vec!["bm25"];

    #[cfg(feature = "fastembed-embeddings")]
    caps.push("semantic");

    #[cfg(feature = "usearch-hnsw")]
    caps.push("hnsw");

    caps
}
```

### Adding New Features

1. Add feature to `Cargo.toml` with optional dependency
2. Gate code with `#[cfg(feature = "...")]`
3. Add error variant for missing feature attempts
4. Update documentation and CI matrix
5. Consider default inclusion based on use case frequency

## References

- [Cargo Features](https://doc.rust-lang.org/cargo/reference/features.html)
- [Conditional Compilation](https://doc.rust-lang.org/reference/conditional-compilation.html)
- [Optional Dependencies](https://doc.rust-lang.org/cargo/reference/features.html#optional-dependencies)
