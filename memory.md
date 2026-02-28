# Perf Improver Memory - rlm-rs

## Last Updated
2026-02-28

## Validated Commands

```bash
cargo test --lib --no-default-features --locked   # Fast offline tests (307 tests, ~0.15s)
cargo clippy --all-targets --no-default-features --locked  # Lint (no errors)
cargo fmt -- --check                              # Format check
cargo build --no-default-features --locked        # Build
# NOTE: --features fastembed-embeddings requires ONNX binary download (blocked in CI sandbox)
```

## Performance Notes

- `semantic_search()` brute-force cosine scan: O(n × D) per query. Rayon parallelization implemented in PR submitted 2026-02-27.
- `buffer_fully_embedded()` was O(n) SQLite queries — replaced with single NOT EXISTS query (PR 2026-02-28).
- HNSW (usearch) is optional feature, disabled by default — sub-linear queries possible for large collections.
- Embeddings: 1024-dim f32 (BGE-M3 model via fastembed), fallback uses hash-based FallbackEmbedder.
- SQLite WAL mode enabled; transactions used for batch inserts.
- `[[bench]]` section is commented out in Cargo.toml — no criterion benchmarks exist yet.

## Round-Robin Task Tracker

| Task | Last Run | Notes |
|------|----------|-------|
| Task 1: Discover Commands | 2026-02-27 | Commands validated |
| Task 2: Identify Opportunities | 2026-02-27 | Backlog populated |
| Task 3: Implement Improvements | 2026-02-28 | buffer_fully_embedded PR submitted |
| Task 4: Maintain PRs | 2026-02-28 | GitHub API unavailable, could not verify PR status |
| Task 5: Comment on Issues | - | Not yet done |
| Task 6: Measurement Infrastructure | - | Not yet done — benchmark suite is priority |
| Task 7: Monthly Summary | 2026-02-28 | Issue created (GitHub API unavailable, new issue created) |

## Optimization Backlog

Priority order:
1. ✅ Parallel cosine similarity (`semantic_search`, `src/search/mod.rs`) — PR submitted 2026-02-27
2. ✅ Pre-size embedding byte buffers (`store_embedding`, `src/storage/sqlite.rs`) — PR submitted 2026-02-27
3. ✅ `buffer_fully_embedded` O(n) SQLite queries → single NOT EXISTS query — PR submitted 2026-02-28 (branch: perf-assist/buffer-fully-embedded-single-query)
4. **NEXT**: Add criterion benchmark suite — `[[bench]]` commented out in Cargo.toml; add benchmarks for semantic_search, buffer_fully_embedded, bm25 search
5. Comment on performance issues (Task 5) — GitHub API unavailable both 2026-02-27 and 2026-02-28

## Work In Progress

None — PR submitted for buffer_fully_embedded optimization.

## Completed Work

- PR 2026-02-27: parallel cosine similarity + Vec::with_capacity for embedding buffers
- PR 2026-02-28: buffer_fully_embedded O(n→1) queries (branch: perf-assist/buffer-fully-embedded-single-query)

## Monthly Summary Issues

- 2026-02: Created via safeoutputs 2026-02-27 (original)
- 2026-02: Created via safeoutputs 2026-02-28 (duplicate — GitHub API unavailable, could not update existing)
  Note: May need maintainer to close duplicate. Prefixed "[Perf Improver] Monthly Activity 2026-02"

## GitHub API Status

- 2026-02-27: API tools unavailable (fetch failed)
- 2026-02-28: API tools still unavailable (fetch failed for list_issues, search_issues, list_pull_requests)
- Workaround: safeoutputs tools work for creating/updating resources
