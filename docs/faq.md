# Frequently Asked Questions (FAQ)

Common questions about rlm-rs and troubleshooting tips.

## General Questions

### What is RLM-RS?

RLM-RS is a CLI tool implementing the Recursive Language Model (RLM) pattern from [arXiv:2512.24601](https://arxiv.org/abs/2512.24601). It enables AI assistants to process documents up to 100x larger than their context window by chunking content and using pass-by-reference architecture.

### How does RLM-RS differ from other chunking tools?

RLM-RS is specifically designed for AI assistant workflows with:
- **Multiple chunking strategies** (semantic, code-aware, fixed, parallel)
- **Automatic embeddings** for semantic search
- **Hybrid search** combining semantic + BM25
- **Pass-by-reference** architecture to reduce token usage
- **SQLite persistence** for reliable state management
- **Claude Code integration** via MCP plugin

### Do I need Claude Code to use RLM-RS?

No, rlm-rs is a standalone CLI tool. However, it's optimized for use with Claude Code via the [rlm-rs plugin](https://github.com/zircote/rlm-plugin).

### What file types does RLM-RS support?

RLM-RS works with any text-based file format:
- **Markdown** (.md) - Use semantic chunker
- **Source code** (.rs, .py, .js, .ts, .go, .java, etc.) - Use code chunker
- **Plain text** (.txt) - Use fixed or semantic chunker
- **Logs** - Use fixed chunker with overlap
- **JSON/YAML** - Use fixed or semantic chunker

Binary files are not supported.

## Installation & Setup

### How do I install RLM-RS?

Three options:

1. **Cargo** (recommended): `cargo install rlm-cli`
2. **Homebrew**: `brew install zircote/tap/rlm-rs`
3. **From source**: See [Getting Started](getting-started.md)

### What are the system requirements?

- **OS**: macOS, Linux, or Windows
- **Rust**: 1.88+ (if building from source)
- **Disk**: ~50MB for binaries + embeddings model
- **Memory**: 512MB minimum, 2GB+ recommended for large files

### Where is the database stored?

By default, RLM-RS creates `.rlm/rlm-state.db` in your current directory.

You can override this with the `RLM_DB_PATH` environment variable:

````bash
export RLM_DB_PATH=/path/to/custom/rlm-state.db
rlm-cli init
````

### How do I reset the database?

````bash
rlm-cli reset
````

**Warning**: This deletes all buffers, chunks, and state. Cannot be undone.

Alternatively, manually delete the database:

````bash
rm -rf .rlm/
````

## Usage Questions

### Which chunking strategy should I use?

| Content Type | Recommended Strategy | Why |
|--------------|---------------------|-----|
| Markdown, documentation | `semantic` | Preserves logical structure (headings, paragraphs) |
| Source code | `code` | Respects function/class boundaries |
| Logs, plain text | `fixed` | Predictable chunk sizes |
| Large files (>10MB) | `parallel` | Faster processing via multi-threading |

**Example**:
````bash
rlm-cli load docs.md --chunker semantic
rlm-cli load src/main.rs --chunker code
rlm-cli load app.log --chunker fixed --chunk-size 150000
````

### How do I choose chunk size?

**Default**: 50,000 bytes (50KB)

**Guidelines**:
- **Smaller chunks** (10-30KB): Better precision, more chunks to search
- **Larger chunks** (50-100KB): Better context, fewer chunks
- **Very large chunks** (100KB+): Risk losing granularity

**Recommendation**: Start with defaults, adjust based on your content.

````bash
rlm-cli load file.txt --chunk-size 30000 --overlap 1000
````

### What's the difference between semantic and BM25 search?

| Search Type | How It Works | Best For |
|-------------|--------------|----------|
| **Semantic** | Finds similar meaning using embeddings | Conceptual queries ("how to install") |
| **BM25** | Finds keyword matches with ranking | Exact terms ("error code 404") |
| **Hybrid** | Combines both via RRF | Most use cases (default) |

**Example**:
````bash
# Semantic search
rlm-cli search "installation process" --mode semantic

# Keyword search
rlm-cli search "error" --mode bm25

# Hybrid (default)
rlm-cli search "database connection error" --mode hybrid
````

### How do I search across multiple buffers?

Omit the `--buffer` flag to search all buffers:

````bash
rlm-cli search "error handling"
````

Or specify multiple buffers:

````bash
# Load files
rlm-cli load src/lib.rs --name lib
rlm-cli load src/main.rs --name main

# Search both
rlm-cli search "parse" --buffer lib --buffer main
````

**Note**: Current implementation searches one buffer at a time when `--buffer` is specified.

### How do I update a buffer without reloading?

Use `update-buffer`:

````bash
rlm-cli update-buffer readme --file README-updated.md
````

This re-chunks and re-embeds the content while preserving the buffer ID.

### Can I export my buffers?

Yes:

````bash
# Export all buffers to JSON
rlm-cli export-buffers > buffers.json

# Export individual buffer content
rlm-cli show readme > readme-content.txt

# Export chunks to individual files
rlm-cli write-chunks readme --output-dir ./chunks/
````

## Performance Questions

### How long does embedding take?

Embedding time depends on:
- **Chunk count**: ~10-50ms per chunk on CPU
- **Model**: BGE-M3 (default) is optimized for CPU
- **Hardware**: Faster on GPU (not yet supported)

**Example**: 100 chunks ≈ 1-5 seconds on modern CPU.

**Tip**: Embeddings are generated automatically during `load` and cached. Re-loading the same content reuses existing embeddings.

### Can I disable embeddings?

Not directly, but you can:
1. Use BM25-only search: `--mode bm25`
2. Build without fastembed: `cargo build --no-default-features`

### How much disk space does RLM-RS use?

- **Database**: Proportional to content size (roughly 1.5-2x source file size)
- **Embeddings**: ~4KB per chunk (1024 dimensions × 4 bytes)
- **Models**: ~150MB for BGE-M3 (downloaded once)

**Example**: 10MB document with 200 chunks ≈ 20MB database + 800KB embeddings.

### How do I handle very large files (>100MB)?

Use parallel chunking:

````bash
rlm-cli load huge-log.txt --chunker parallel --chunk-size 100000
````

**Tips**:
- Use `--chunk-size` to control memory usage
- Consider splitting files externally if >1GB
- Use `grep` for keyword search instead of loading entire file

## Troubleshooting

### "RLM not initialized" error

You need to initialize the database first:

````bash
rlm-cli init
````

This creates `.rlm/rlm-state.db`.

### Embeddings fail with "model download error"

**Cause**: Network issue or insufficient disk space.

**Solutions**:
1. Check network connection
2. Verify disk space (~150MB needed)
3. Try manual download: models are cached in `~/.cache/rlm-rs/`
4. Build without embeddings: `cargo build --no-default-features`

### "Buffer not found" error

**Cause**: Buffer name or ID doesn't exist.

**Solution**: List buffers to verify:

````bash
rlm-cli list
````

### Search returns no results

**Possible causes**:
1. **No embeddings**: Check `rlm-cli chunk status`
2. **Wrong buffer**: Verify with `rlm-cli list`
3. **Query mismatch**: Try different search mode or keywords

**Debug**:
````bash
# Check embedding status
rlm-cli chunk status

# Try keyword search
rlm-cli search "your query" --mode bm25

# Try broader query
rlm-cli search "error" --top-k 20
````

### High memory usage

**Causes**:
- Large chunk sizes
- Many buffers loaded
- Memory-mapped files not released

**Solutions**:
- Use smaller `--chunk-size`
- Delete unused buffers: `rlm-cli delete <buffer>`
- Restart CLI to release memory maps

### Slow search performance

**Solutions**:
1. Enable HNSW: Build with `--features usearch-hnsw`
2. Reduce search space: Use `--buffer` to limit scope
3. Use BM25-only: `--mode bm25` (faster than semantic)
4. Reduce `--top-k`: Fewer results = faster search

## Integration Questions

### How do I use RLM-RS with Claude Code?

Install the [rlm-rs MCP plugin](https://github.com/zircote/rlm-plugin):

1. Add to `.vscode/mcp.json` or `~/.config/Claude/claude_desktop_config.json`
2. Restart Claude Code
3. Use RLM commands in conversations

See [Plugin Integration](plugin-integration.md) for details.

### Can I use RLM-RS with other AI assistants?

Yes! RLM-RS is a standard CLI tool. Integration requires:
1. Ability to execute shell commands
2. Parsing JSON output (use `--format json`)
3. Managing state (buffer IDs, chunk IDs)

### Can I call RLM-RS from scripts?

Absolutely:

````bash
#!/bin/bash
rlm-cli init
rlm-cli load document.md --name doc --format json > load-result.json
chunk_id=$(jq '.chunks[0].id' load-result.json)
rlm-cli chunk get "$chunk_id"
````

Use `--format json` for machine-readable output.

## Development Questions

### How do I build from source?

````bash
git clone https://github.com/zircote/rlm-rs.git
cd rlm-rs
cargo build --release
# Binary at: target/release/rlm-cli
````

### What are the feature flags?

| Feature | Description | Default |
|---------|-------------|---------|
| `fastembed-embeddings` | BGE-M3 semantic embeddings | ✅ Enabled |
| `usearch-hnsw` | HNSW approximate search | ❌ Disabled |
| `full-search` | Both embeddings + HNSW | ❌ Disabled |

See [Features Guide](features.md) for details.

### How do I contribute?

See [CONTRIBUTING.md](../CONTRIBUTING.md) for:
- Development setup
- Code style guidelines
- Testing requirements
- PR process

### Where are the tests?

- **Unit tests**: `src/**/*.rs` (`#[cfg(test)]` modules)
- **Integration tests**: `tests/integration_test.rs`
- **Property tests**: Using `proptest` for property-based testing

Run tests:
````bash
cargo test
````

## See Also

- **[Getting Started](getting-started.md)** - Quick start tutorial
- **[Troubleshooting](troubleshooting.md)** - Detailed troubleshooting guide
- **[CLI Reference](cli-reference.md)** - Complete command reference
- **[Examples](examples.md)** - Practical examples and workflows

---

**Last Updated**: 2026-02-18  
**Version**: 1.2.4

**Still have questions?** 
- Open an issue: [GitHub Issues](https://github.com/zircote/rlm-rs/issues)
- Start a discussion: [GitHub Discussions](https://github.com/zircote/rlm-rs/discussions)
