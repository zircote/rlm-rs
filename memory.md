# RLM-RS Test Improver Memory

## Build/Test Commands (Validated)

- **Test (no fastembed - works in CI sandbox)**: `cargo test --no-default-features --locked`
- **Test (full, needs network for ONNX download)**: `cargo test --features fastembed-embeddings --locked`
- **Lint (strict)**: `cargo clippy --all-targets --all-features --locked -- -D warnings`
- **Format**: `cargo fmt`
- **Format check**: `cargo fmt -- --check`
- **Build**: `cargo build --all-features --locked`

**Notes**: 
- `fastembed-embeddings` feature requires downloading ONNX RT binaries at build time (network needed)
- Pre-existing clippy errors in `src/search/hnsw.rs` (missing `# Errors` docs, `const fn`) - NOT new issues

## Testing Notes

- Tests use `#[allow(clippy::expect_used)]` or `.unwrap()` in test modules
- SQLite storage tests use `SqliteStorage::in_memory()` helper
- Test framework: Rust built-in + `proptest` + `test-case`
- MSRV: 1.88

## Testing Backlog (Opportunities)

1. **[DONE - PR submitted 2026-02-27]** `SqliteStorage` embedding functions - 0 tests, 23 added
2. RRF/weighted_rrf edge cases (empty weighted list, NaN/infinity scores)
3. `src/chunking/semantic.rs` - 29 tests but complex Unicode logic may have gaps
4. `src/chunking/code.rs` - 14 tests, language-aware chunking logic
5. Integration tests: FTS search + embedding pipeline end-to-end

## Work In Progress / Completed

### 2026-02-27 - Run 1
- **Task 1**: Validated build commands. `cargo test --no-default-features --locked` = 88 tests pass
- **Task 3**: Added 23 embedding tests to `src/storage/sqlite.rs` (PR submitted as draft)
- **Task 7**: Created Monthly Activity Summary issue

## Tasks Last Run (Round-Robin)

- Task 1 (commands): 2026-02-27
- Task 3 (implement tests): 2026-02-27
- Task 7 (monthly summary): 2026-02-27

## Maintainer Priorities

No specific priorities communicated yet.

## Previously Checked Off Items

(none)
