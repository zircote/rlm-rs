---
title: "Error Handling with thiserror"
description: "Standardized error handling strategy using thiserror for type-safe, no-panic library code"
type: adr
category: architecture
tags:
  - error-handling
  - thiserror
  - no-panic
  - api-design
status: accepted
created: 2025-01-27
updated: 2025-01-27
author: zircote
project: rlm-rs
technologies:
  - rust
  - thiserror
audience:
  - developers
  - contributors
related:
  - 002-use-rust-as-implementation-language
  - 005-cli-first-interface-design
---

# ADR-011: Error Handling with thiserror

## Status

Accepted

## Context

### Background and Problem Statement

Error handling in Rust libraries requires careful consideration of several factors: type safety, error context preservation, API ergonomics, and panic avoidance. For a CLI tool that may be integrated into AI assistant workflows, predictable error behavior is critical. AI assistants need structured error information to provide recovery guidance.

### Current Approach

RLM-RS uses `thiserror` for deriving error types with the following characteristics:

1. **Hierarchical error types**: Top-level `Error` enum with domain-specific variants (`StorageError`, `IoError`, `ChunkingError`, `CommandError`)
2. **No panics in library code**: All fallible operations return `Result<T, Error>`
3. **Source chain preservation**: Errors wrap underlying causes for debugging
4. **Structured error output**: JSON format includes error type and recovery suggestions

## Decision Drivers

### Primary Decision Drivers

1. **No-panic guarantee**: Library code must never panic, enabling safe integration
2. **Error traceability**: Preserve full error chain for debugging
3. **API ergonomics**: Clean error types that work with `?` operator
4. **AI integration**: Structured errors enable programmatic error handling

### Secondary Decision Drivers

1. **Compile-time safety**: Exhaustive matching on error variants
2. **Minimal boilerplate**: Derive macros reduce manual implementation
3. **Standard compatibility**: Implement `std::error::Error` trait

## Considered Options

### Option 1: thiserror (Chosen)

Derive macro for implementing `std::error::Error` with minimal boilerplate.

**Pros:**
- Zero runtime cost
- Automatic `Display`, `Error`, and `From` implementations
- Source chain preservation via `#[source]`
- Works seamlessly with `?` operator

**Cons:**
- Procedural macro adds compile time
- Less flexible than manual implementation

### Option 2: anyhow

Type-erased error handling for applications.

**Pros:**
- Simple API with automatic context attachment
- Good for applications where error types aren't part of API

**Cons:**
- Type erasure loses compile-time error variant checking
- Not suitable for library APIs where callers need to match on errors

### Option 3: Manual Implementation

Hand-written `Error` trait implementations.

**Pros:**
- Full control over implementation details
- No macro dependencies

**Cons:**
- Significant boilerplate
- Error-prone manual implementations
- Harder to maintain

## Decision

We chose **thiserror** for error handling with the following patterns:

### Error Hierarchy

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("I/O error: {0}")]
    Io(#[from] IoError),

    #[error("chunking error: {0}")]
    Chunking(#[from] ChunkingError),

    #[error("command error: {0}")]
    Command(#[from] CommandError),
}
```

### No-Panic Policy

Library code uses these patterns instead of panicking:

| Panic Pattern | Safe Alternative |
|---------------|------------------|
| `.unwrap()` | `.ok_or(Error::...)? ` |
| `.expect()` | `.ok_or_else(\|\| Error::...)?` |
| `panic!()` | `return Err(Error::...)` |
| `unreachable!()` | Match all variants or use `_` |

### Structured JSON Errors

```json
{
  "success": false,
  "error": {
    "type": "BufferNotFound",
    "message": "storage error: buffer not found: main",
    "suggestion": "Run 'rlm-rs list' to see available buffers"
  }
}
```

## Consequences

### Positive Consequences

1. **Predictable behavior**: No unexpected panics in production
2. **Better debugging**: Full error chain available
3. **Type safety**: Exhaustive matching catches missing error handlers
4. **AI-friendly**: Structured errors enable automated recovery

### Negative Consequences

1. **Verbose match arms**: Must handle all error variants
2. **Conversion boilerplate**: Manual `From` implementations for some types

### Neutral Consequences

1. **Slightly longer compile times**: Procedural macros add overhead
2. **Learning curve**: Contributors must understand error propagation

## Implementation Notes

### Adding New Error Variants

1. Add variant to appropriate error enum
2. Implement `Display` message via `#[error("...")]`
3. Add `#[from]` if auto-conversion from source is desired
4. Update `get_error_details()` in `output.rs` for JSON formatting

### Testing Error Paths

```rust
#[test]
fn test_error_on_missing_buffer() {
    let storage = create_test_storage();
    let result = storage.get_buffer("nonexistent");
    assert!(matches!(result, Err(Error::Storage(StorageError::BufferNotFound { .. }))));
}
```

## References

- [thiserror crate](https://docs.rs/thiserror)
- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [RustConf 2020: Error handling Isn't All About Errors](https://www.youtube.com/watch?v=rAF8mLI0naQ)
