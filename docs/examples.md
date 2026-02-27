# Examples

This guide provides practical examples for common `rlm-cli` workflows.

## Table of Contents

1. [Basic CLI Usage](#basic-cli-usage)
2. [Chunking Strategies](#chunking-strategies)
3. [Search Workflows](#search-workflows)
4. [Claude Code Integration](#claude-code-integration)
5. [Agentic Workflows](#agentic-workflows)
6. [Rust Library Usage](#rust-library-usage)
7. [Advanced Patterns](#advanced-patterns)

---

## Basic CLI Usage

### Initialize and Load a Document

````bash
# Initialize database
rlm-cli init

# Load a markdown file with semantic chunking
rlm-cli load README.md --name readme --chunker semantic

# Check status
rlm-cli status

# Output:
# Database: .rlm/rlm-state.db (256 KB)
# Buffers: 1
# Total chunks: 47
# Embedded chunks: 47 (100%)
````

### Search and Retrieve

````bash
# Hybrid search (semantic + BM25)
rlm-cli search "installation instructions" --buffer readme --mode hybrid --top-k 5

# Output:
# Chunk ID: 12 | Score: 0.89 | Buffer: readme
# ## Installation
# 
# ### Via Cargo (Recommended)
# cargo install rlm-cli
# ...

# Retrieve specific chunk by ID
rlm-cli chunk get 12

# Regex search for specific patterns
rlm-cli grep readme "cargo install" --context 2
````

---

## Chunking Strategies

### Semantic Chunking (Markdown/Documentation)

Best for: Markdown, documentation, prose

````bash
# Load with semantic boundaries (headings, paragraphs)
rlm-cli load docs/architecture.md \
  --name architecture \
  --chunker semantic \
  --chunk-size 150000 \
  --overlap 1000

# Chunk boundaries respect:
# - Markdown headings (##, ###)
# - Paragraph breaks
# - Code blocks
# - List boundaries
````

**Typical output:**
- Chunk 1: Introduction section
- Chunk 2: Architecture Overview section
- Chunk 3: Module Structure section

### Code-Aware Chunking (Source Files)

Best for: Source code (Rust, Python, JavaScript, etc.)

````bash
# Load Rust source with code-aware chunking
rlm-cli load src/main.rs \
  --name main-source \
  --chunker code

# Splits at function/struct/impl boundaries
````

**Supported languages:**
- Rust, Python, JavaScript, TypeScript
- Go, Java, C/C++, Ruby, PHP

**Example chunk boundaries:**

````rust
// Chunk 1: Imports + struct definition
use std::io;
pub struct Config { ... }

// Chunk 2: First impl block
impl Config {
    pub fn new() -> Self { ... }
}

// Chunk 3: Second impl block
impl Default for Config { ... }
````

### Fixed Chunking (Logs/Plain Text)

Best for: Log files, plain text, structured data

````bash
# Fixed-size chunks with overlap
rlm-cli load logs/server.log \
  --name server-logs \
  --chunker fixed \
  --chunk-size 100000 \
  --overlap 500

# Splits at exact byte boundaries
# Overlap ensures no context loss at chunk edges
````

### Parallel Chunking (Large Files)

Best for: Large files (>10MB), multi-core systems

````bash
# Parallel chunking for speed
rlm-cli load dataset.txt \
  --name large-dataset \
  --chunker parallel \
  --chunk-size 200000

# Uses all CPU cores (Rayon thread pool)
# Typically 3-5x faster than sequential chunking
````

**Performance example:**

| File Size | Sequential | Parallel (8 cores) |
|-----------|------------|-------------------|
| 10 MB | 2.5s | 0.8s |
| 100 MB | 25s | 6s |
| 1 GB | 4m 10s | 1m 2s |

---

## Search Workflows

### Hybrid Search (Semantic + BM25)

Combines semantic similarity and keyword matching using RRF (Reciprocal Rank Fusion):

````bash
# Hybrid search with RRF fusion
rlm-cli search "error handling patterns" \
  --buffer main-source \
  --mode hybrid \
  --top-k 10

# Scores combine:
# - Semantic similarity (cosine distance)
# - BM25 keyword relevance
# - RRF fusion (k=60)
````

**Use cases:**
- Finding conceptually similar content with keyword relevance
- Robust search when terminology varies
- Best balance of precision and recall

### Semantic-Only Search

Pure vector similarity search:

````bash
# Semantic search only
rlm-cli search "how to configure the database" \
  --buffer architecture \
  --mode semantic \
  --top-k 5

# Uses cosine similarity of BGE-M3 embeddings
# Good for: conceptual matches, paraphrased queries
````

### BM25-Only Search

Keyword-based full-text search:

````bash
# BM25 keyword search
rlm-cli search "SQLite initialization" \
  --buffer architecture \
  --mode bm25 \
  --top-k 5

# Uses FTS5 full-text index
# Good for: exact terms, technical jargon, code identifiers
````

### Search with Filters

````bash
# Search specific buffer
rlm-cli search "async" --buffer main-source

# List all chunks in a buffer
rlm-cli chunk list --buffer main-source

# Get embedding status
rlm-cli chunk status --buffer main-source
````

---

## Claude Code Integration

### Complete RLM Workflow

**Scenario:** Analyze a large codebase with Claude Code

**Step 1:** Load the codebase

````bash
# Initialize rlm-rs
rlm-cli init

# Load each source directory
rlm-cli load src/ --name source-code --chunker code
rlm-cli load tests/ --name test-code --chunker code
rlm-cli load docs/ --name documentation --chunker semantic

# Check status
rlm-cli status
# Output: 3 buffers, 542 chunks, 100% embedded
````

**Step 2:** Search and retrieve context

````bash
# Find error handling patterns
rlm-cli search "error handling" \
  --buffer source-code \
  --mode hybrid \
  --top-k 10 \
  --format json > results.json

# Get specific chunks by ID
rlm-cli chunk get 127 --format json
````

**Step 3:** Dispatch to subagents

````bash
# Split chunks into batches for parallel analysis
rlm-cli dispatch source-code \
  --batch-size 5 \
  --task "Analyze error handling patterns" \
  --format json > batches.json

# Each batch contains chunk IDs for subagent processing
````

**Step 4:** Aggregate results

````bash
# After subagent analysis, combine findings
rlm-cli aggregate \
  --findings findings1.json findings2.json findings3.json \
  --output summary.json
````

### Using with Claude Code MCP Plugin

Install the [rlm-rs MCP plugin](https://github.com/zircote/rlm-plugin):

````bash
# Configure Claude Code
cat > ~/.config/claude-code/mcp.json <<EOF
{
  "mcpServers": {
    "rlm": {
      "command": "rlm-mcp-server",
      "args": ["--db-path", ".rlm/rlm-state.db"]
    }
  }
}
EOF

# Start Claude Code - rlm-rs tools are now available
````

**Available MCP tools:**
- `rlm_load` - Load files into buffers
- `rlm_search` - Hybrid search
- `rlm_chunk_get` - Retrieve chunks by ID
- `rlm_dispatch` - Create subagent batches

---

## Agentic Workflows

### Orchestrator → Analyst → Synthesizer Pattern

**Architecture:**

````
┌──────────────┐
│ Orchestrator │  Root LLM (Opus/Sonnet)
└──────┬───────┘
       │ dispatch
       ▼
┌──────────────┐
│  Analysts    │  Sub-LLMs (Haiku) - Process batches
│  (parallel)  │
└──────┬───────┘
       │ findings
       ▼
┌──────────────┐
│ Synthesizer  │  Root LLM - Aggregate results
└──────────────┘
````

**Implementation:**

````bash
# 1. Orchestrator: Create analysis batches
rlm-cli dispatch codebase \
  --batch-size 10 \
  --task "Find security vulnerabilities" \
  --output batches.json

# batches.json:
# [
#   {"batch_id": 1, "chunks": [1,2,3,4,5,6,7,8,9,10]},
#   {"batch_id": 2, "chunks": [11,12,13,14,15,16,17,18,19,20]},
#   ...
# ]

# 2. Analyst: Process each batch (parallel)
for batch in $(jq -r '.[] | @base64' batches.json); do
  BATCH_ID=$(echo $batch | base64 -d | jq -r '.batch_id')
  CHUNK_IDS=$(echo $batch | base64 -d | jq -r '.chunks[]')
  
  # Retrieve chunks
  for chunk_id in $CHUNK_IDS; do
    rlm-cli chunk get $chunk_id >> "batch_${BATCH_ID}_content.txt"
  done
  
  # Analyze with sub-LLM (Haiku)
  analyze_security "batch_${BATCH_ID}_content.txt" > "findings_${BATCH_ID}.json"
done

# 3. Synthesizer: Aggregate findings
rlm-cli aggregate findings_*.json --output final_report.json
````

### Prompt Templates

See `docs/prompts/` for reference:

- **[rlm-orchestrator.md](prompts/rlm-orchestrator.md)** - Root LLM prompt for task decomposition
- **[rlm-analyst.md](prompts/rlm-analyst.md)** - Sub-LLM prompt for chunk analysis
- **[rlm-synthesizer.md](prompts/rlm-synthesizer.md)** - Root LLM prompt for result aggregation

---

## Rust Library Usage

### Basic Initialization

````rust
use rlm_rs::{
    storage::{SqliteStorage, Storage},
    core::Buffer,
    chunking::{SemanticChunker, Chunker},
    error::Result,
};

fn main() -> Result<()> {
    // Initialize storage
    let mut storage = SqliteStorage::new(".rlm/rlm-state.db")?;
    storage.create_schema()?;
    
    // Create buffer
    let content = std::fs::read_to_string("document.md")?;
    let buffer = Buffer::new("docs", content);
    
    // Chunk content
    let chunker = SemanticChunker::new(150_000, 1000);
    let chunks = chunker.chunk(&buffer)?;
    
    println!("Created {} chunks", chunks.len());
    Ok(())
}
````

### Embedding and Search

````rust
use rlm_rs::{
    embedding::{FastEmbedEmbedder, Embedder},
    search::hybrid_search,
    storage::{SqliteStorage, Storage},
};

fn search_example() -> Result<()> {
    let mut storage = SqliteStorage::new(".rlm/rlm-state.db")?;
    
    // Initialize embedder (requires fastembed-embeddings feature)
    #[cfg(feature = "fastembed-embeddings")]
    {
        let embedder = FastEmbedEmbedder::new()?;
        
        // Embed query
        let query_embedding = embedder.embed("error handling")?;
        
        // Hybrid search
        let results = hybrid_search(
            &mut storage,
            "error handling",
            &query_embedding,
            "codebase",
            10, // top-k
        )?;
        
        for result in results {
            println!("Chunk {}: {:.2}", result.chunk_id, result.score);
        }
    }
    
    Ok(())
}
````

### Custom Chunking Strategy

````rust
use rlm_rs::{
    chunking::{Chunker, traits::ChunkerTrait},
    core::{Buffer, Chunk},
    error::Result,
};

struct CustomChunker {
    delimiter: String,
}

impl ChunkerTrait for CustomChunker {
    fn chunk(&self, buffer: &Buffer) -> Result<Vec<Chunk>> {
        let parts: Vec<&str> = buffer.content()
            .split(&self.delimiter)
            .collect();
        
        let chunks = parts
            .into_iter()
            .enumerate()
            .map(|(idx, content)| {
                Chunk::new(
                    idx as i64,
                    buffer.name().to_string(),
                    content.to_string(),
                    idx * content.len(),
                )
            })
            .collect();
        
        Ok(chunks)
    }
}

fn main() -> Result<()> {
    let buffer = Buffer::new("data", "part1|||part2|||part3".to_string());
    let chunker = CustomChunker {
        delimiter: "|||".to_string(),
    };
    
    let chunks = chunker.chunk(&buffer)?;
    println!("Created {} chunks", chunks.len());
    Ok(())
}
````

---

## Advanced Patterns

### Incremental Updates

Update buffer content and re-embed only changed chunks:

````bash
# Initial load
rlm-cli load document.md --name docs

# Modify document.md externally
# ...

# Update buffer (only re-embeds changed chunks)
rlm-cli update-buffer docs document-updated.md

# Force re-embedding all chunks
rlm-cli chunk embed --buffer docs --force
````

### Multi-Buffer Workflows

Work with multiple document sources:

````bash
# Load multiple sources
rlm-cli load api-docs.md --name api-docs --chunker semantic
rlm-cli load source-code/ --name source --chunker code
rlm-cli load tests/ --name tests --chunker code

# Search across all buffers
rlm-cli search "authentication" --top-k 10

# Search specific buffer
rlm-cli search "authentication" --buffer api-docs

# Export all buffers
rlm-cli export-buffers --output all-buffers.json
````

### Context Variables

Use variables for dynamic context management:

````bash
# Set context variable
rlm-cli var set current_task "security-audit"

# Get variable
rlm-cli var get current_task
# Output: security-audit

# List all variables
rlm-cli var list

# Set global variable (persistent across sessions)
rlm-cli global set project_name "rlm-rs"
````

### Regex Search with Context

````bash
# Search with context lines
rlm-cli grep source-code "fn main" \
  --context 5 \
  --max-matches 10

# Output:
# Buffer: source-code | Match: 1
# ----
# use std::io;
# use clap::Parser;
# 
# fn main() {
#     let args = Cli::parse();
#     // ...
# }
# ----

# Case-insensitive search
rlm-cli grep docs "error" --ignore-case
````

### JSON Output for Scripting

````bash
# JSON output for programmatic use
rlm-cli search "async" --buffer source --format json | jq '.results[0]'

# Output:
# {
#   "chunk_id": 42,
#   "buffer_name": "source",
#   "score": 0.87,
#   "content": "async fn process_data() { ... }",
#   "start_offset": 12500
# }

# Chain with other tools
rlm-cli chunk list --buffer source --format json \
  | jq '.chunks | length'
# Output: 127
````

### Peek at Buffer Content

````bash
# View first 3000 characters
rlm-cli peek docs --start 0 --end 3000

# View middle section
rlm-cli peek docs --start 10000 --end 15000

# View from offset to end
rlm-cli peek docs --start 50000
````

### Write Chunks to Files

````bash
# Export each chunk to separate files
rlm-cli write-chunks source-code --output-dir ./chunks/

# Result:
# chunks/
# ├── chunk_0001.txt
# ├── chunk_0002.txt
# ├── chunk_0003.txt
# ...
````

---

## Performance Tips

### Chunking Performance

````bash
# For large files, use parallel chunking
rlm-cli load large-file.txt \
  --chunker parallel \
  --chunk-size 100000

# Reduce chunk size for faster embedding
rlm-cli load docs.md \
  --chunker semantic \
  --chunk-size 50000  # Smaller chunks = faster embedding
````

### Search Performance

````bash
# Use smaller top-k for faster results
rlm-cli search "query" --top-k 5  # vs --top-k 100

# Use BM25-only for faster keyword search
rlm-cli search "exact term" --mode bm25

# Use semantic-only for concept search
rlm-cli search "general idea" --mode semantic
````

### Memory Management

````bash
# For very large files, increase chunk size to reduce memory
rlm-cli load huge-file.txt \
  --chunker parallel \
  --chunk-size 500000  # Larger chunks = fewer in memory

# Export and delete old buffers
rlm-cli export-buffers --output backup.json
rlm-cli delete old-buffer
````

---

## Troubleshooting

### Slow Embedding Generation

**Symptom:** `rlm-cli load` takes minutes to complete

**Solutions:**
````bash
# 1. Use parallel chunking
rlm-cli load file.txt --chunker parallel

# 2. Increase chunk size (fewer chunks to embed)
rlm-cli load file.txt --chunk-size 200000

# 3. Check CPU usage (should be 100% across all cores)
top -p $(pgrep rlm-cli)
````

### Out of Memory Errors

**Symptom:** `rlm-cli search` crashes with OOM

**Solutions:**
````bash
# 1. Reduce top-k
rlm-cli search "query" --top-k 5  # Instead of 100

# 2. Delete unused buffers
rlm-cli list
rlm-cli delete unused-buffer

# 3. Rebuild without HNSW
cargo build --release --features fastembed-embeddings
````

### Missing Embeddings

**Symptom:** Semantic search returns empty results

**Solutions:**
````bash
# 1. Check embedding status
rlm-cli chunk status --buffer docs

# 2. Generate embeddings if missing
rlm-cli chunk embed --buffer docs

# 3. Force re-embedding
rlm-cli chunk embed --buffer docs --force
````

---

## Next Steps

- **[Features Guide](features.md)** - Learn about feature flags and optimization
- **[CLI Reference](cli-reference.md)** - Complete command documentation
- **[Architecture](architecture.md)** - Understand internal design
- **[Plugin Integration](plugin-integration.md)** - Integrate with Claude Code and other tools
