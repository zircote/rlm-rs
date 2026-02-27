# Perf Improver Memory - rlm-rs

## Last Updated
2026-02-27

## Validated Commands

```bash
cargo test --lib --no-default-features --locked   # Fast offline tests (287 tests, ~0.12s)
cargo clippy --all-targets --no-default-features --locked  # Lint (no errors)
cargo fmt -- --check                              # Format check
cargo build --no-default-features --locked        # Build
# NOTE: --features fastembed-embeddings requires ONNX binary download (blocked in CI sandbox)
```

## Performance Notes

- `semantic_search()` brute-force cosine scan: O(n × D) per query. Rayon parallelization implemented in PR #perf-assist/parallel-cosine-similarity-and-embed-alloc.
- HNSW (usearch) is optional feature, disabled by default — sub-linear queries possible for large collections.
- Embeddings: 1024-dim f32 (BGE-M3 model via fastembed), fallback uses hash-based FallbackEmbedder.
- SQLite WAL mode enabled; transactions used for batch inserts.

## Round-Robin Task Tracker

| Task | Last Run | Notes |
|------|----------|-------|
| Task 1: Discover Commands | 2026-02-27 | Commands validated |
| Task 2: Identify Opportunities | 2026-02-27 | Backlog populated |
| Task 3: Implement Improvements | 2026-02-27 | PR submitted |
| Task 4: Maintain PRs | - | First run |
| Task 5: Comment on Issues | - | First run |
| Task 6: Measurement Infrastructure | - | First run |
| Task 7: Monthly Summary | 2026-02-27 | Issue created |

## Optimization Backlog

Priority order:
1. ✅ Parallel cosine similarity (`semantic_search`, `src/search/mod.rs`) — DONE
2. ✅ Pre-size embedding byte buffers (`store_embedding`, `src/storage/sqlite.rs`) — DONE
3. `buffer_fully_embedded` O(n) SQLite queries → single COUNT(*) query
4. Benchmark suite: `[[bench]]` commented out in Cargo.toml — add criterion benchmarks
5. HNSW default: usearch feature disabled by default; large-collection users benefit from enabling

## Work In Progress

Branch: `perf-assist/parallel-cosine-similarity-and-embed-alloc`
Status: PR submitted (patch captured via safeoutputs)
Changes: par_iter in semantic_search + Vec::with_capacity for embedding buffers

## Completed Work

(None yet — first run)

## Monthly Summary Issues

- 2026-02: Created (safeoutputs) — issue title "[Perf Improver] Monthly Activity 2026-02"
