# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **Chunking**: `MAX_CHUNK_SIZE` lowered from 50,000 to 24,000 bytes —
  token-dense text can run well under the ~4 bytes/token assumed by the old ceiling,
  so chunks under 50,000 bytes could still exceed BGE-M3's 8192-token limit and get
  silently truncated on embed; the new ceiling (~8k tokens at ~3 bytes/token for
  ASCII-heavy text) reduces the risk of truncation ([#313](https://github.com/zircote/rlm-rs/issues/313))

### Added

- **Tooling**: project-local `/release` Claude Code skill
  (`.claude/skills/release/`) — orchestrates the full attested release
  (`/release v<X.Y.Z>` or `/release patch|minor|major`): prep PR, tag,
  chain monitoring, failure playbook, and independent workstation
  verification

## [1.3.1] - 2026-06-12

### Fixed

- **Release**: Homebrew formula updates now trigger via `workflow_run` on
  the Release workflow — releases are authored by `github-actions[bot]`
  (the tap token is scoped to the tap repo only), and bot-authored release
  events never trigger workflows, so the formula silently stopped updating;
  the formula commit is also idempotent now that two triggers are wired

### Added

- **Release**: the published crates.io `.crate` archive now carries SLSA
  build provenance — the workflow downloads the registry-served bytes,
  asserts they match the locally packaged crate, and attests them;
  verification instructions added to `SECURITY.md`

## [1.3.0] - 2026-06-12

### Fixed

- **Site**: restore docs site build — Starlight 0.39 removed labeled
  `autogenerate` sidebar groups; the "Workflow Reference" group now nests
  the autogenerate config in `items` (broken since the auto-merged
  Starlight 0.40 bump, which no CI gate builds the site to catch)
- **Tests**: embedding tests skip gracefully when the BGE-M3 model cannot
  be downloaded (network-dependent Hugging Face fetch) instead of failing
  the suite — this intermittently broke CI on main

### Added

- **CI**: `site` job builds the docs site on every push/PR and gates
  `All Checks Pass` — site-dependency bumps can no longer automerge with
  a broken build
- **Release**: Attested delivery — releases now publish platform binaries
  (`rlm-cli-{version}-{platform}` for linux-amd64, linux-arm64, macos-arm64,
  windows-amd64.exe), each carrying SLSA build provenance via
  `actions/attest-build-provenance`
  - macos-amd64 is not published: ort (onnxruntime, via the default
    fastembed feature) provides no prebuilt binaries for
    `x86_64-apple-darwin`
  - Release publication is gated on fail-closed attestation verification —
    a tag publishes nothing unattested
  - `workflow_dispatch` dry-run exercises the build → attest → verify chain
    without cutting a release
- **Security**: `SECURITY.md` with vulnerability reporting policy and
  release-artifact verification instructions (`gh attestation verify`)
- **Release**: CycloneDX SBOM per release (Syft), attested to every binary
  via `actions/attest-sbom` and verified fail-closed alongside provenance
- **Release**: `cargo audit` gate — a release publishes nothing if any
  dependency carries a known vulnerability
- **Release**: test gate — tags are not guaranteed to point at CI-green
  commits, so the release workflow runs the full test suite and publishes
  nothing untested
- **Release**: `rlm-cli-{version}-checksums.txt` asset with SHA-256 digests
  of all release artifacts
- **Release**: crates.io publishing now uses Trusted Publishing (OIDC) via
  `rust-lang/crates-io-auth-action` instead of a long-lived registry token
  (requires one-time Trusted Publishing setup on crates.io before the next
  release)

### Security

- **Dependencies**: update npm `uuid` to 14.0.0 in the docs site lockfile
  (CVE-2026-41907 / GHSA-w5hq-g745-h8pq, fixed ≥ 11.1.1)
- **Dependencies**: update quinn-proto 0.11.13 → 0.11.14 in the lockfile
  (RUSTSEC-2026-0037); the entry was an unreachable phantom resolution — no
  shipped binary included the vulnerable code — but the audit gate flags it
- **Supply chain**: align `deny.toml` `[graph] targets` with the release
  matrix (adds `aarch64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`) so
  cargo-deny analyzes every shipped platform's dependency graph
- **CI**: `pin-check` job (central `zircote/.github` reusable workflow)
  asserts every GitHub Actions `uses:` reference is pinned to a full
  40-character commit SHA
- **CI**: pinned the remaining mutable workflow ref
  (`reusable-dependabot-automerge.yml@main`) to a commit SHA

### Changed

- **Build**: Bump MSRV to 1.95 — libsqlite3-sys 0.38.1 (via the rusqlite
  0.40.1 bump) uses `cfg_select!`, stabilized in Rust 1.95; the MSRV CI job
  now also passes `--locked` so it checks the dependency versions that
  actually ship

- **Documentation**: Enhanced troubleshooting guide with usearch version pinning details
  - Documented the `<2.25` version constraint; usearch v2.24.x is supported since rlm-cli v1.2.4
  - Added troubleshooting section for usearch version issues
  - Updated features guide to reference usearch v2.23.x requirement

### Performance

- **Search**: Batch chunk lookup in `populate_previews()` using `get_chunks_by_ids()`
  - Replaces N individual `get_chunk()` calls with a single `WHERE id IN (…)` query
  - Reduces database round-trips from O(n) to O(1) when rendering content previews
- **Search**: Parallelize cosine similarity scan in `semantic_search()` using `rayon::par_iter()`
  - Linear scan over stored embeddings now distributes across all available CPU cores
  - Near-linear speedup for large embedding collections (>1 000 chunks)
- **Storage**: Pre-size embedding byte buffers in `store_embedding()` and `store_embeddings_batch()`
  - `Vec::with_capacity(embedding.len() * 4)` avoids repeated heap reallocations per embedding
  - Reduces heap churn when indexing large batches of chunks
- **Storage**: Replace O(n) per-chunk embedding checks with a single `NOT EXISTS` query
  - `SqliteStorage::all_chunks_have_embeddings()` and `buffer_fully_embedded()` now issue one SQL
    query regardless of how many chunks a buffer contains
  - Eliminates the previous pattern of issuing one `has_embedding()` call per chunk when checking
    whether a buffer is fully embedded

## [1.2.4] - 2026-02-08

### Added

- **Chunking**: Code-aware chunker for language-specific boundaries
  - Supports Rust, Python, JavaScript, TypeScript, Go, Java, C/C++, Ruby, PHP
  - Splits at function, class, and method boundaries
  - Available via `--chunker code` or `--chunker ast`
- **Search**: HNSW vector index for scalable approximate nearest neighbor search
  - O(log n) search performance
  - Optional feature: enable with `usearch-hnsw` feature flag
- **Search**: Content preview in search results with `--preview` flag
  - Configurable length with `--preview-len` (default: 150 chars)
- **CLI**: `update-buffer` command to update buffer content with re-chunking
  - Supports `--embed` flag for automatic re-embedding
  - Incremental embedding (only new/changed chunks)
- **CLI**: `dispatch` command for parallel subagent processing
  - Split chunks into batches by size or worker count
  - Filter chunks by search query
- **CLI**: `aggregate` command to combine analyst findings
  - Filter by relevance level
  - Group and sort findings
  - Store results in output buffer
- **Embedding**: Incremental embedding support
  - Only embeds new or changed chunks
  - Model version tracking for migration detection
- **Embedding**: Model name tracking in `Embedder` trait
- **Output**: NDJSON format support (`--format ndjson`)
- **CI**: Crates.io publish workflow with pre-publish validation
- **CI**: Dependabot auto-merge workflow
- **Documentation**: ADRs for error handling, concurrency model, and feature flags
- **Documentation**: MCP agentic workflow prompts (analyst, orchestrator, synthesizer)

### Changed

- **Core**: Consolidated UTF-8 and timestamp utilities in io module
  - `find_char_boundary` and `current_timestamp` now shared across modules
- **Core**: Improved token estimation with `estimate_tokens_accurate()` method
- **Error**: Dedicated `Embedding` error variant in `StorageError`
- **Embedding**: Removed unnecessary unsafe `Send`/`Sync` impls from `FallbackEmbedder`

### Fixed

- **Search**: Resolve usearch segfault via move semantics fix
- **Documentation**: Fix unresolved intra-doc links in Chunk methods

### Security

- Update `bytes` to 1.11.1 to resolve RUSTSEC-2026-0007

### Dependencies

- Bump `bytes` from 1.10.1 to 1.11.1
- Bump `clap` from 4.5.54 to 4.5.56 ([#11])
- Bump the github-actions group with 2 updates ([#12])
- Bump `actions/github-script` from 7 to 8 ([#7])
- Bump `criterion` from 0.5.1 to 0.8.1 ([#9])
- Bump `rusqlite` from 0.33.0 to 0.38.0 ([#8])
- Bump `actions/checkout` from 4 to 6 ([#6])
- Bump `taiki-e/install-action` in the github-actions group ([#5])

## [1.2.3] - 2026-01-20

### Fixed

- **CI**: Allow `multiple_crate_versions` lint (fastembed transitive deps)
- **CI**: Add ISC, BSD, MPL-2.0, CDLA-Permissive-2.0 to allowed licenses
- **CI**: Ignore unmaintained `paste` advisory (fastembed transitive dep)
- **CI**: Skip openssl ban check for fastembed transitive deps

## [1.2.2] - 2026-01-20

### Fixed

- **CLI**: Handle broken pipe gracefully when output is piped to commands like `jq` or `head`

## [1.2.1] - 2026-01-20

### Fixed

- **Build**: Enable `fastembed-embeddings` feature by default (BGE-M3 now works out of the box)

## [1.2.0] - 2026-01-20

### Added

- **Search**: Search results now include `index` (document position) and `buffer_id` fields for temporal ordering
- **Documentation**: Architecture Decision Records (ADRs) documenting 10 key architectural decisions from project history

### Changed

- **Embedding**: Switch from all-MiniLM-L6-v2 to BGE-M3 embedding model
  - Dimensions increased from 384 to 1024 for richer semantic representation
  - Token context increased from ~512 to 8192 for full chunk coverage
  - **Breaking**: Existing embeddings must be regenerated (schema migration v3 clears old embeddings)
- **Build**: Bump MSRV to 1.88

### Fixed

- **Search**: Escape FTS5 special characters in search queries to prevent syntax errors
- **Chunking**: Validate UTF-8 boundaries in semantic chunker search window to prevent panics on multi-byte characters

## [1.1.2] - 2026-01-19

### Changed

- **Chunking**: Reduced default chunk size from 240,000 to 3,000 characters for better semantic search granularity
- **Chunking**: Reduced max chunk size from 250,000 to 50,000 characters

## [1.1.1] - 2026-01-19

### Fixed

- **Search**: BM25 scores now display in scientific notation for small values (e.g., `1.60e-6` instead of `0.0000`)
- **Search**: FTS queries use OR semantics for multi-word searches (more forgiving matching)
- **Embedding**: Auto-embedding during load now outputs proper JSON when `--format json` is used

## [1.1.0] - 2026-01-19

### Added

- **Search**: Hybrid semantic + BM25 search with Reciprocal Rank Fusion (RRF)
- **Search**: `search` command with `--mode` option (`hybrid`, `semantic`, `bm25`)
- **Embedding**: Auto-embedding during `load` command (embeddings generated automatically)
- **Chunks**: `chunk get` command for pass-by-reference retrieval
- **Chunks**: `chunk list` command to list chunks for a buffer
- **Chunks**: `chunk embed` command to generate/regenerate embeddings
- **Chunks**: `chunk status` command to show embedding status

### Changed

- **Load**: Embeddings are now generated automatically during load (no separate embed step needed)

## [1.0.0] - 2026-01-19

### Added

- **Core**: Initial release with semantic search and pass-by-reference architecture
- **Chunking**: Fixed, semantic, and parallel chunking strategies
- **Storage**: SQLite persistence for buffers, chunks, and variables
- **Search**: Regex search with `grep` command
- **I/O**: Memory-mapped file handling for large documents
- **CLI**: JSON output format support for all commands

## [0.2.0] - 2026-01-19

### Added

- **CI/CD**: Release workflow to auto-update Homebrew tap

## [0.1.0] - 2026-01-19

### Added

- Initial implementation of RLM-RS CLI
- Buffer management (load, list, show, delete, peek)
- Chunking with configurable strategies
- Variable storage (context and global)
- Export functionality

[Unreleased]: https://github.com/zircote/rlm-rs/compare/v1.3.1...HEAD
[1.3.1]: https://github.com/zircote/rlm-rs/compare/v1.3.0...v1.3.1
[1.3.0]: https://github.com/zircote/rlm-rs/compare/v1.2.4...v1.3.0
[1.2.4]: https://github.com/zircote/rlm-rs/compare/v1.2.3...v1.2.4
[1.2.3]: https://github.com/zircote/rlm-rs/compare/v1.2.2...v1.2.3
[1.2.2]: https://github.com/zircote/rlm-rs/compare/v1.2.1...v1.2.2
[1.2.1]: https://github.com/zircote/rlm-rs/compare/v1.2.0...v1.2.1
[1.2.0]: https://github.com/zircote/rlm-rs/compare/v1.1.2...v1.2.0
[1.1.2]: https://github.com/zircote/rlm-rs/compare/v1.1.1...v1.1.2
[1.1.1]: https://github.com/zircote/rlm-rs/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/zircote/rlm-rs/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/zircote/rlm-rs/compare/v0.2.0...v1.0.0
[0.2.0]: https://github.com/zircote/rlm-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/zircote/rlm-rs/releases/tag/v0.1.0
