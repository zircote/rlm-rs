//! Integration tests for RLM-RS.

#![allow(clippy::expect_used)]

use rlm_rs::core::{Buffer, Chunk, Context, ContextValue};
use rlm_rs::storage::{SqliteStorage, Storage};
use tempfile::TempDir;

/// Helper to create a test storage instance.
fn create_test_storage() -> (SqliteStorage, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut storage = SqliteStorage::open(&db_path).expect("Failed to create storage");
    storage.init().expect("Failed to init storage");
    (storage, temp_dir)
}

#[test]
fn test_storage_init_and_status() {
    let (storage, _temp) = create_test_storage();

    assert!(storage.is_initialized().expect("is_initialized failed"));

    let stats = storage.stats().expect("stats failed");
    assert_eq!(stats.buffer_count, 0);
    assert_eq!(stats.chunk_count, 0);
}

#[test]
fn test_buffer_crud() {
    let (mut storage, _temp) = create_test_storage();

    // Create buffer
    let buffer = Buffer::from_named(
        "test-buffer".to_string(),
        "Test content for the buffer".to_string(),
    );

    let id = storage.add_buffer(&buffer).expect("add_buffer failed");
    assert!(id > 0);

    // Read buffer by ID
    let loaded = storage.get_buffer(id).expect("get_buffer failed");
    assert!(loaded.is_some());
    let loaded = loaded.expect("buffer should exist");
    assert_eq!(loaded.content, "Test content for the buffer");

    // Read buffer by name
    let by_name = storage
        .get_buffer_by_name("test-buffer")
        .expect("get_buffer_by_name failed");
    assert!(by_name.is_some());

    // List buffers
    let buffers = storage.list_buffers().expect("list_buffers failed");
    assert_eq!(buffers.len(), 1);

    // Delete buffer
    storage.delete_buffer(id).expect("delete_buffer failed");
    let deleted = storage
        .get_buffer(id)
        .expect("get_buffer after delete failed");
    assert!(deleted.is_none());
}

#[test]
fn test_chunks() {
    let (mut storage, _temp) = create_test_storage();

    // Create buffer first
    let buffer = Buffer::from_content("Hello, world! This is test content.".to_string());
    let buffer_id = storage.add_buffer(&buffer).expect("add_buffer failed");

    // Create chunks
    let chunks = vec![
        Chunk::new(buffer_id, "Hello, world!".to_string(), 0..13, 0),
        Chunk::new(buffer_id, " This is test content.".to_string(), 13..35, 1),
    ];

    storage
        .add_chunks(buffer_id, &chunks)
        .expect("add_chunks failed");

    // Retrieve chunks
    let loaded_chunks = storage.get_chunks(buffer_id).expect("get_chunks failed");
    assert_eq!(loaded_chunks.len(), 2);
    assert_eq!(loaded_chunks[0].content, "Hello, world!");
    assert_eq!(loaded_chunks[1].index, 1);

    // Count chunks
    let count = storage.chunk_count(buffer_id).expect("chunk_count failed");
    assert_eq!(count, 2);
}

#[test]
fn test_context_operations() {
    let (mut storage, _temp) = create_test_storage();

    // Initially no context
    let ctx = storage.load_context().expect("load_context failed");
    assert!(ctx.is_none());

    // Create and save context
    let mut context = Context::new();
    context.set_variable(
        "key1".to_string(),
        ContextValue::String("value1".to_string()),
    );
    context.set_global(
        "global_key".to_string(),
        ContextValue::String("global_value".to_string()),
    );

    storage.save_context(&context).expect("save_context failed");

    // Load context back
    let loaded = storage.load_context().expect("load_context failed");
    assert!(loaded.is_some());
    let loaded = loaded.expect("context should exist");

    assert_eq!(
        loaded.get_variable("key1"),
        Some(&ContextValue::String("value1".to_string()))
    );
    assert_eq!(
        loaded.get_global("global_key"),
        Some(&ContextValue::String("global_value".to_string()))
    );
}

#[test]
fn test_chunker_strategies() {
    use rlm_rs::chunking::{Chunker, FixedChunker, available_strategies, create_chunker};

    // Test available strategies
    let strategies = available_strategies();
    assert!(strategies.contains(&"fixed"));
    assert!(strategies.contains(&"semantic"));
    assert!(strategies.contains(&"parallel"));

    // Test creating chunkers by name
    let fixed = create_chunker("fixed");
    assert!(fixed.is_ok());

    let semantic = create_chunker("semantic");
    assert!(semantic.is_ok());

    let unknown = create_chunker("unknown");
    assert!(unknown.is_err());

    // Test chunking
    let content = "Line one.\nLine two.\nLine three.";
    let chunker = FixedChunker::with_size(15);
    let chunks = chunker.chunk(1, content, None).expect("chunk failed");
    assert!(!chunks.is_empty());
}

#[test]
fn test_storage_reset() {
    let (mut storage, _temp) = create_test_storage();

    // Add some data
    let buffer = Buffer::from_content("content".to_string());
    storage.add_buffer(&buffer).expect("add_buffer failed");

    let stats = storage.stats().expect("stats failed");
    assert_eq!(stats.buffer_count, 1);

    // Reset
    storage.reset().expect("reset failed");

    let stats = storage.stats().expect("stats after reset failed");
    assert_eq!(stats.buffer_count, 0);
}

mod property_tests {
    use proptest::prelude::*;
    use rlm_rs::core::Chunk;

    proptest! {
        #[test]
        fn chunk_size_matches_content(content in "[a-z]{1,100}") {
            let chunk = Chunk::new(1, content.clone(), 0..content.len(), 0);
            prop_assert_eq!(chunk.size(), content.len());
        }

        #[test]
        fn chunk_byte_range_valid(start in 0usize..1000, len in 1usize..100) {
            let content = "x".repeat(len);
            let end = start + len;
            let chunk = Chunk::new(1, content, start..end, 0);
            prop_assert_eq!(chunk.start(), start);
            prop_assert_eq!(chunk.end(), end);
            prop_assert_eq!(chunk.range_size(), len);
        }

        #[test]
        fn chunk_estimate_tokens_reasonable(content in "[a-z ]{1,200}") {
            let chunk = Chunk::new(1, content.clone(), 0..content.len(), 0);
            let tokens = chunk.estimate_tokens();
            // Approximately 4 chars per token
            let expected_min = content.len() / 6;
            let expected_max = content.len() / 2;
            prop_assert!(tokens >= expected_min || content.len() < 4);
            prop_assert!(tokens <= expected_max + 1);
        }
    }
}
