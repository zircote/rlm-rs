# Perf Improver Memory - rlm-rs

## Last Updated
2026-03-01

## Validated Commands

```bash
cargo build --no-default-features --locked       # Build (26s cold, 1s warm)
cargo test --lib --no-default-features --locked   # Test (310 tests, ~0.13s)
cargo clippy --all-targets --no-default-features --locked  # Lint
cargo fmt -- --check                              # Format check
# NOTE: --features fastembed-embeddings requires ONNX binary download (blocked in CI sandbox)
```

## Round-Robin Task Tracker

| Task | Last Run | Notes |
|------|----------|-------|
| Task 1: Discover Commands | 2026-03-01 | Commands validated |
| Task 2: Identify Opportunities | 2026-03-01 | Backlog populated |
| Task 3: Implement Improvements | 2026-03-01 | PR created for populate_previews N+1 fix |
| Task 4: Maintain PRs | - | Not yet done |
| Task 5: Comment on Issues | - | Not yet done |
| Task 6: Perf Infrastructure | - | Not yet done |
| Task 7: Monthly Summary | 2026-03-01 | Issue created |

## Performance Opportunities Backlog

Priority order:
1. ✅ **DONE** batch chunk lookup in `populate_previews` — replaced N get_chunk() calls with 1 WHERE IN query (PR submitted 2026-03-01)
2. **NEXT**: Batch chunk lookup in `hybrid_search` — `from_chunk_id` is called N times too (each calls `get_chunk`). Could add a `from_chunk_ids_batch()` approach.
3. Reduce string allocations in `search_fts` — `format!("\"{}\"", term)` allocates per-term; use a pre-sized String builder.
4. `store_embeddings_batch` — reuse a single `Vec<u8>` buffer across iterations instead of allocating per embedding (`Vec::with_capacity(embedding.len() * 4)` per loop).
5. Add criterion benchmarks for search path — currently no bench/ directory; adding benchmarks would make future perf work evidence-based.

## Performance Notes

- SQLite is in-memory for tests (`SqliteStorage::in_memory()`), on-disk uses WAL mode
- `rusqlite::params_from_iter` works with `ids.iter().copied()` for dynamic IN clauses
- Dynamic SQL for IN clauses: build `format!("WHERE id IN ({})", "?,".repeat(n))` — safe for i64 IDs (not user input)
- `populate_previews` is called in CLI search display path — latency directly user-visible
- No criterion benches exist yet; synthetic loop measurements used instead

## Completed Work

- 2026-03-01: PR for batch chunk lookup in populate_previews (branch: perf-assist/batch-chunk-lookup)

## Monthly Summary Issues

- 2026-03: Created via safeoutputs 2026-03-01
