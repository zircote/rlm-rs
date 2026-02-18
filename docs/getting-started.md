# Getting Started with RLM-RS

This guide will get you up and running with rlm-rs in 5 minutes.

## What is RLM-RS?

RLM-RS is a CLI tool that helps AI assistants (like Claude Code) process documents that are too large for their context window. It does this by:

1. **Chunking** - Breaking large documents into smaller pieces
2. **Embedding** - Creating semantic representations for search
3. **Searching** - Finding relevant chunks using hybrid semantic + keyword search
4. **Passing by Reference** - Letting AI agents retrieve specific chunks by ID

Think of it as a "smart pagination system" that lets AI assistants navigate huge documents efficiently.

## Prerequisites

- **Rust 1.88+** (or use pre-built binaries)
- **macOS, Linux, or Windows**

## Installation

### Option 1: Cargo (Recommended)

````bash
cargo install rlm-rs
````

### Option 2: Homebrew (macOS/Linux)

````bash
brew tap zircote/tap
brew install rlm-rs
````

### Option 3: From Source

````bash
git clone https://github.com/zircote/rlm-rs.git
cd rlm-rs
make install
````

### Verify Installation

````bash
rlm-rs --version
# Output: rlm-cli 1.2.4
````

## Your First RLM Session

### Step 1: Initialize the Database

RLM-RS stores state in a local SQLite database:

````bash
rlm-rs init
````

This creates `.rlm/rlm-state.db` in your current directory.

### Step 2: Load a Document

Let's load a markdown file with automatic chunking and embedding:

````bash
rlm-rs load README.md --name readme --chunker semantic
````

This:
- Loads `README.md` into a buffer named "readme"
- Chunks it at natural boundaries (headings, paragraphs)
- Generates semantic embeddings automatically

### Step 3: Check Status

````bash
rlm-rs status
````

Output:
````
Database: .rlm/rlm-state.db (256 KB)
Buffers: 1
Total chunks: 47
Embedded chunks: 47 (100%)
````

### Step 4: Search the Content

Use hybrid search (semantic + BM25):

````bash
rlm-rs search "installation instructions" --buffer readme --top-k 3
````

Output:
````
Chunk ID: 12 | Score: 0.89 | Buffer: readme
## Installation

### Via Cargo (Recommended)
cargo install rlm-rs
...
````

### Step 5: Retrieve by ID

Once you know a chunk ID, you can retrieve it directly:

````bash
rlm-rs chunk get 12
````

This is the "pass-by-reference" pattern - instead of copying text, you pass chunk IDs.

## Understanding Chunking Strategies

RLM-RS supports multiple chunking strategies:

| Strategy | Best For | How It Works |
|----------|----------|--------------|
| `semantic` | Markdown, documentation | Splits at headings and paragraphs |
| `code` | Source code | Splits at function/class boundaries |
| `fixed` | Plain text, logs | Fixed-size chunks with overlap |
| `parallel` | Large files (>10MB) | Multi-threaded fixed chunking |

### Example: Code-Aware Chunking

````bash
rlm-rs load src/main.rs --name maincode --chunker code
````

This splits Rust code at function boundaries, keeping functions intact.

### Example: Fixed Chunking with Overlap

````bash
rlm-rs load large-log.txt --chunker fixed --chunk-size 150000 --overlap 1000
````

## Common Workflows

### Workflow 1: Analyzing a Large Codebase

````bash
# Load multiple files
rlm-rs load src/lib.rs --name lib --chunker code
rlm-rs load src/main.rs --name main --chunker code
rlm-rs load tests/integration.rs --name tests --chunker code

# Search across all buffers
rlm-rs search "error handling" --top-k 10

# View specific chunks
rlm-rs chunk get 42
````

### Workflow 2: Processing Large Documents

````bash
# Load with semantic chunking
rlm-rs load whitepaper.md --name paper --chunker semantic

# Use regex search for specific terms
rlm-rs grep paper "performance|benchmark" --max-matches 20

# Peek at specific sections
rlm-rs peek paper --start 0 --end 5000
````

### Workflow 3: Parallel Subagent Processing

````bash
# Dispatch chunks to multiple AI agents
rlm-rs dispatch --buffer paper --batch-size 5

# (After subagent analysis)
# Aggregate findings
rlm-rs aggregate
````

## Integration with Claude Code

RLM-RS is designed to work with the [rlm-rs Claude Code plugin](https://github.com/zircote/rlm-plugin).

The RLM architecture:
- **Root LLM**: Main Claude conversation (Opus/Sonnet)
- **Sub-LLM**: Analyst agents (Haiku) via `rlm-subcall`
- **External Environment**: `rlm-rs` CLI + SQLite

See [Plugin Integration](plugin-integration.md) for details.

## Tips for Effective Usage

1. **Choose the Right Chunker**: Semantic for prose, code for source files, fixed for logs
2. **Use Descriptive Buffer Names**: `--name docs` instead of auto-generated names
3. **Leverage Hybrid Search**: Combines semantic understanding with keyword precision
4. **Pass by Reference**: Share chunk IDs instead of copying large text blocks
5. **Clean Up**: Use `rlm-rs delete <buffer>` or `rlm-rs reset` to manage state

## Next Steps

Now that you're up and running:

1. **Explore Commands**: See the [CLI Reference](cli-reference.md)
2. **Try Examples**: Work through [Examples](examples.md)
3. **Customize Features**: Learn about [Feature Flags](features.md)
4. **Get Help**: Check [Troubleshooting](troubleshooting.md) if you hit issues

## Getting Help

- **FAQ**: [Frequently Asked Questions](faq.md)
- **Troubleshooting**: [Common Issues](troubleshooting.md)
- **Issues**: [GitHub Issues](https://github.com/zircote/rlm-rs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/zircote/rlm-rs/discussions)

---

**Next**: [Examples](examples.md) | [CLI Reference](cli-reference.md) | [Troubleshooting](troubleshooting.md)
