# Test Improver Memory - rlm-rs

## Last Updated
2026-02-28

## Validated Commands

```bash
cargo test --lib --no-default-features --locked   # Fast offline tests (318 tests after this run, ~0.17s)
cargo clippy --all-targets --no-default-features --locked  # Lint
cargo fmt -- --check                              # Format check
cargo build --no-default-features --locked        # Build
# NOTE: --features fastembed-embeddings requires ONNX binary download (blocked in CI sandbox)
```

## Testing Notes

- Tests use in-memory SQLite (`SqliteStorage::in_memory()`) for isolation - no tempfiles needed
- `FallbackEmbedder::new(DEFAULT_DIMENSIONS)` is the go-to embedder for tests (no model download)
- Test helpers `setup_storage()` and `setup_storage_with_chunks()` exist in `src/search/mod.rs`
- Clippy runs with pedantic+nursery lints: any `unused_mut` in test code will warn
- GitHub API tools (list_issues, list_pull_requests, create_branch) all fail with `fetch failed` in this environment
- `safeoutputs-create_pull_request` works and handles pushing the branch

## Round-Robin Task Tracker

| Task | Last Run | Notes |
|------|----------|-------|
| Task 1: Discover Commands | 2026-02-28 | Commands validated |
| Task 2: Identify Opportunities | 2026-02-28 | Backlog populated |
| Task 3: Implement Improvements | 2026-02-28 | PR created for search/mod.rs untested functions |
| Task 4: Maintain PRs | - | Not yet done |
| Task 5: Comment on Issues | - | Not yet done |
| Task 6: Test Infrastructure | - | Not yet done |
| Task 7: Monthly Summary | 2026-02-28 | Issue created |

## Testing Backlog

Priority order:
1. ✅ Tests for check_model_mismatch, get_embedding_model_info, populate_previews, IncrementalEmbedResult edge cases — PR submitted 2026-02-28 (branch: test-assist/search-mod-untested-functions)
2. **NEXT**: Tests for `src/chunking/code.rs` - CodeChunker has language detection, regex-based code boundary detection; has 14 existing tests but language detection path (detect_language fn) could use more coverage
3. Tests for `src/cli/commands.rs` - has 12 tests but many complex code paths (embed, search, chunk commands)
4. Tests for `src/io/reader.rs` - 25 tests exist, check if mmap large file path is covered

## Work In Progress

None — PR submitted for search/mod.rs untested functions.

## Completed Work

- 2026-02-28: PR for check_model_mismatch, get_embedding_model_info, populate_previews, IncrementalEmbedResult (branch: test-assist/search-mod-untested-functions)

## Monthly Summary Issues

- 2026-02: Created via safeoutputs 2026-02-28 (Test Improver Monthly Activity 2026-02)
