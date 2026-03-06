# Troubleshooting Guide

Common issues and solutions when working with `rlm-cli`.

## Table of Contents

1. [Installation Issues](#installation-issues)
2. [Build Failures](#build-failures)
3. [Runtime Errors](#runtime-errors)
4. [Performance Issues](#performance-issues)
5. [Search Problems](#search-problems)
6. [Embedding Issues](#embedding-issues)
7. [Database Issues](#database-issues)

---

## Installation Issues

### Cargo Install Fails

**Symptom:**

````
error: failed to compile `rlm-cli` v1.2.4
````

**Common causes:**

1. **Rust version too old**

````bash
# Check version
rustc --version
# Required: 1.88+

# Update Rust
rustup update stable
````

2. **Missing C++ compiler (usearch-hnsw feature)**

````bash
# Ubuntu/Debian
sudo apt-get install build-essential

# macOS
xcode-select --install

# Or install without HNSW
cargo install rlm-cli --no-default-features --features fastembed-embeddings
````

3. **Network issues downloading dependencies**

````bash
# Use vendored dependencies
cargo install rlm-cli --locked

# Or retry with verbose output
cargo install rlm-cli -v
````

---

## Build Failures

### ONNX Runtime Link Error

**Symptom:**

````
error: linking with `cc` failed
undefined reference to `onnxruntime_*`
````

**Solution:**

FastEmbed uses bundled ONNX binaries by default (no action needed). If you see this error, ensure you're not overriding default features:

````bash
# Correct - uses bundled ONNX
cargo build --release

# Incorrect - disables bundled binaries
cargo build --release --features fastembed-embeddings --no-default-features
````

### usearch Compile Error

**Symptom:**

````
error: failed to compile usearch-sys
C++ compilation failed
````

**Solutions:**

1. **Install C++ compiler:**

````bash
# Ubuntu/Debian
sudo apt-get install g++ clang

# macOS
xcode-select --install
````

2. **Build without HNSW:**

````bash
cargo build --release --features fastembed-embeddings
````

3. **Check C++ version (requires C++17):**

````bash
g++ --version  # Should be 7.0+
clang++ --version  # Should be 5.0+
````

### usearch Version Issues

**Background:**

rlm-cli uses usearch 2.24.0 from crates.io, pinned to `<2.25`.

**Why version 2.25+ is not yet included:**

usearch v2.24.0 is now supported since rlm-cli v1.2.4 shipped a move-semantics fix that resolved the segfault. The upper bound `<2.25` is a conservative pin pending validation of newer releases. See [unum-cloud/USearch#715](https://github.com/unum-cloud/USearch/issues/715).

**If you encounter version-related errors:**

1. **Verify Cargo.lock uses 2.24.x:**

````bash
grep -A2 'name = "usearch"' Cargo.lock
````

Expected output should show version `2.24.x`.

2. **Clear cache and rebuild:**

````bash
cargo clean
rm -rf ~/.cargo/registry/cache/
cargo build --release --features usearch-hnsw
````

3. **Check for git dependencies:**

Ensure `Cargo.toml` references the official crates.io version, not a git fork:

````toml
usearch = { version = ">=2.23, <2.25", optional = true }
````

**Not** a git dependency like:

````toml
# INCORRECT - do not use git forks
usearch = { git = "https://github.com/...", branch = "..." }
````

### Clippy Warnings Blocking Build

**Symptom:**

````
error: unwrap_used
  --> src/main.rs:42:18
   |
42 |     let x = y.unwrap();
   |                ^^^^^^
````

**Solution:**

This is expected - clippy is configured to deny unwraps. Fix by using `?` or proper error handling:

````rust
// Before
let x = y.unwrap();

// After
let x = y.map_err(|e| Error::Custom(e.to_string()))?;
````

---

## Runtime Errors

### Database File Not Found

**Symptom:**

````
Error: Database file not found: .rlm/rlm-state.db
````

**Solution:**

Initialize the database:

````bash
rlm-cli init
````

Or specify custom path:

````bash
rlm-cli --db-path /path/to/db.sqlite init
rlm-cli --db-path /path/to/db.sqlite status
````

Set environment variable for persistent custom path:

````bash
export RLM_DB_PATH=/path/to/db.sqlite
rlm-cli status
````

### Permission Denied

**Symptom:**

````
Error: Permission denied (os error 13)
````

**Solutions:**

1. **Check directory permissions:**

````bash
ls -la .rlm/
chmod 755 .rlm/
chmod 644 .rlm/rlm-state.db
````

2. **Use custom path with write access:**

````bash
mkdir -p ~/rlm-data
rlm-cli --db-path ~/rlm-data/rlm.db init
````

### Buffer Not Found

**Symptom:**

````
Error: Buffer 'docs' not found
````

**Solutions:**

1. **List available buffers:**

````bash
rlm-cli list
````

2. **Load the buffer if missing:**

````bash
rlm-cli load document.md --name docs
````

3. **Check for typos in buffer name (case-sensitive):**

````bash
# Wrong
rlm-cli search "query" --buffer Docs

# Correct
rlm-cli search "query" --buffer docs
````

---

## Performance Issues

### Slow Load Times

**Symptom:**

Loading a 100MB file takes >5 minutes

**Solutions:**

1. **Use parallel chunking:**

````bash
# Before: Sequential chunking
rlm-cli load large.txt --chunker fixed

# After: Parallel chunking
rlm-cli load large.txt --chunker parallel
````

2. **Increase chunk size (fewer chunks to embed):**

````bash
# Before: Many small chunks
rlm-cli load file.txt --chunk-size 50000  # More chunks

# After: Fewer large chunks
rlm-cli load file.txt --chunk-size 200000  # Fewer chunks
````

3. **Disable embedding if not needed:**

````bash
# Build without embeddings
cargo build --release --no-default-features

# Then load without embedding overhead
rlm-cli load file.txt
````

**Performance comparison (100MB file):**

| Configuration | Time | Chunks |
|---------------|------|--------|
| Sequential, 50KB chunks | 4m 30s | 2000 |
| Parallel, 50KB chunks | 1m 15s | 2000 |
| Parallel, 200KB chunks | 25s | 500 |

### Slow Search Times

**Symptom:**

Search takes >10 seconds for 50K chunks

**Solutions:**

1. **Enable HNSW vector index:**

````bash
# Rebuild with HNSW support
cargo build --release --features full-search

# Search will use approximate NN (much faster)
rlm-cli search "query" --buffer docs
````

2. **Reduce top-k:**

````bash
# Before
rlm-cli search "query" --top-k 100  # Slow

# After
rlm-cli search "query" --top-k 10  # Much faster
````

3. **Use BM25-only for keyword search:**

````bash
# Semantic search is slower
rlm-cli search "exact keyword" --mode hybrid

# BM25-only is faster
rlm-cli search "exact keyword" --mode bm25
````

**Search performance (50K chunks):**

| Mode | Without HNSW | With HNSW |
|------|--------------|-----------|
| BM25 | 200ms | 200ms |
| Semantic (exact) | 5000ms | 5000ms |
| Semantic (HNSW) | N/A | 8ms |
| Hybrid | 5200ms | 220ms |

### High Memory Usage

**Symptom:**

````
rlm-cli process using 8GB RAM
````

**Solutions:**

1. **Reduce chunk count:**

````bash
# Increase chunk size
rlm-cli load file.txt --chunk-size 500000  # Larger chunks
````

2. **Delete unused buffers:**

````bash
rlm-cli list
rlm-cli delete old-buffer-1
rlm-cli delete old-buffer-2
````

3. **Use BM25-only (no embedding memory):**

````bash
# Rebuild without embeddings
cargo build --release --no-default-features
````

4. **Disable HNSW index (saves ~4x memory):**

````bash
cargo build --release --features fastembed-embeddings
````

**Memory estimates:**

| Configuration | 10K chunks | 50K chunks | 100K chunks |
|---------------|------------|------------|-------------|
| BM25-only | 50MB | 200MB | 400MB |
| + Embeddings (exact) | 250MB | 1.2GB | 2.4GB |
| + HNSW index | 450MB | 2.2GB | 4.4GB |

---

## Search Problems

### Empty Search Results

**Symptom:**

````bash
rlm-cli search "query" --buffer docs
# No results found
````

**Solutions:**

1. **Check buffer exists:**

````bash
rlm-cli list
````

2. **Check buffer has chunks:**

````bash
rlm-cli chunk list docs
````

3. **Check embeddings are generated:**

````bash
rlm-cli chunk status
# Shows embedding status for all buffers
````

4. **Try different search mode:**

````bash
# Try BM25 keyword search
rlm-cli search "query" --buffer docs --mode bm25

# Try semantic-only
rlm-cli search "query" --buffer docs --mode semantic
````

5. **Check query spelling:**

````bash
# Typo
rlm-cli search "errro handling"  # No results

# Correct
rlm-cli search "error handling"  # Results found
````

### Poor Search Relevance

**Symptom:**

Search returns irrelevant results

**Solutions:**

1. **Use hybrid search for best results:**

````bash
# Better: Combines semantic + keyword
rlm-cli search "authentication flow" --mode hybrid
````

2. **Increase top-k to see more results:**

````bash
rlm-cli search "query" --top-k 20  # Instead of default 10
````

3. **Try different query phrasing:**

````bash
# Too specific
rlm-cli search "JWT authentication with refresh tokens"

# More general
rlm-cli search "authentication tokens"
````

4. **Check chunk boundaries:**

````bash
# View chunk content
rlm-cli chunk get 42

# Might need different chunking strategy
rlm-cli delete docs
rlm-cli load document.md --name docs --chunker semantic
````

---

## Embedding Issues

### Embeddings Not Generated

**Symptom:**

````bash
rlm-cli chunk status
# Embedded: 0/100 (0%)
````

**Solutions:**

1. **Generate embeddings manually:**

````bash
rlm-cli chunk embed docs
````

2. **Check fastembed feature is enabled:**

````bash
rlm-cli --version
# Should show: Features: fastembed-embeddings
````

3. **Rebuild with embeddings:**

````bash
cargo build --release --features fastembed-embeddings
````

### Model Download Fails

**Symptom:**

````
Error: Failed to download embedding model
Network error: Connection timeout
````

**Solutions:**

1. **Check internet connection:**

````bash
curl -I https://huggingface.co/
````

2. **Retry download (model cached after first success):**

````bash
rm -rf ~/.cache/fastembed/
rlm-cli chunk embed docs
````

3. **Use HTTP proxy if needed:**

````bash
export HTTP_PROXY=http://proxy.example.com:8080
export HTTPS_PROXY=http://proxy.example.com:8080
rlm-cli chunk embed docs
````

4. **Download model manually:**

````bash
# Download BGE-M3 model manually
mkdir -p ~/.cache/fastembed/BAAI__bge-m3
# Copy model files to this directory
````

### Embedding Dimension Mismatch

**Symptom:**

````
Error: Embedding dimension mismatch: expected 1024, got 384
````

**Cause:**

Buffer was embedded with a different model (e.g., all-MiniLM-L6-v2 vs BGE-M3)

**Solution:**

Re-embed with current model:

````bash
rlm-cli chunk embed docs --force
````

---

## Database Issues

### Database Locked

**Symptom:**

````
Error: database is locked
````

**Solutions:**

1. **Close other rlm-cli processes:**

````bash
# Check for running processes
ps aux | grep rlm-cli

# Kill if needed
pkill rlm-cli
````

2. **Wait for lock to release:**

SQLite locks are temporary - wait 5-10 seconds and retry.

3. **Check for stale lock file:**

````bash
# Remove .rlm directory entirely (WARNING: deletes all data)
rm -rf .rlm/
rlm-cli init
````

### Database Corruption

**Symptom:**

````
Error: database disk image is malformed
````

**Solutions:**

1. **Check database integrity:**

````bash
sqlite3 .rlm/rlm-state.db "PRAGMA integrity_check;"
````

2. **Export data before recovery:**

````bash
# Export buffers if possible
rlm-cli export-buffers --output backup.json
````

3. **Reset database (last resort - DESTROYS DATA):**

````bash
rm .rlm/rlm-state.db
rlm-cli init
````

4. **Restore from backup:**

````bash
# If you have backup.json from export-buffers
# Manually re-load documents
````

### Disk Space Issues

**Symptom:**

````
Error: No space left on device
````

**Solutions:**

1. **Check database size:**

````bash
rlm-cli status
# Shows: Database: .rlm/rlm-state.db (512 MB)
````

2. **Delete unused buffers:**

````bash
rlm-cli list
rlm-cli delete old-buffer
````

3. **Vacuum database:**

````bash
sqlite3 .rlm/rlm-state.db "VACUUM;"
````

4. **Check disk space:**

````bash
df -h .
````

---

## Getting Help

If these solutions don't resolve your issue:

1. **Check existing issues:** [GitHub Issues](https://github.com/zircote/rlm-rs/issues)

2. **Enable verbose output:**

````bash
rlm-cli --verbose <command>
````

3. **Collect diagnostic info:**

````bash
# Version and features
rlm-cli --version

# Database status
rlm-cli status

# System info
uname -a
rustc --version
````

4. **Open an issue:** [New Issue](https://github.com/zircote/rlm-rs/issues/new)

Include:
- `rlm-cli --version` output
- Operating system and Rust version
- Full error message
- Steps to reproduce

---

## Related Documentation

- [Features Guide](features.md) - Understanding feature flags and build options
- [Examples](examples.md) - Usage examples and workflows
- [CLI Reference](cli-reference.md) - Complete command documentation
- [Architecture](architecture.md) - Internal design and implementation
