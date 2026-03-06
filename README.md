# rlm-rs

[![CI](https://github.com/zircote/rlm-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/zircote/rlm-rs/actions/workflows/ci.yml)
[![Rust Version](https://img.shields.io/badge/rust-1.88%2B-dea584?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)

Recursive Language Model (RLM) CLI for Claude Code - handles long-context tasks via chunking and recursive sub-LLM calls.

Based on the RLM pattern from [arXiv:2512.24601](https://arxiv.org/abs/2512.24601), enabling analysis of documents up to 100x larger than typical context windows.

## Features

- **Hybrid Semantic Search**: Combined semantic + BM25 search with RRF fusion
- **Auto-Embedding**: Embeddings generated automatically during load (BGE-M3 model)
- **Pass-by-Reference**: Retrieve chunks by ID for efficient subagent processing
- **Multiple Chunking Strategies**: Fixed, semantic, code-aware, and parallel chunking
- **Code-Aware Chunking**: Language-aware chunking at function/class boundaries
- **HNSW Vector Index**: Optional scalable approximate nearest neighbor search
- **Incremental Embedding**: Efficient partial re-embedding for updated content
- **Agentic Workflow Support**: dispatch/aggregate commands for parallel subagent processing
- **SQLite State Persistence**: Reliable buffer management across sessions
- **Regex Search**: Fast content search with context windows
- **Memory-Mapped I/O**: Efficient handling of large files
- **JSON/NDJSON Output**: Machine-readable output for integration

## How It Works

<p align="center">
  <img src=".github/readme-infographic.svg" alt="RLM Architecture Diagram" width="800">
</p>

## Installation

### Via Cargo (Recommended)

```bash
cargo install rlm-cli
```

### Via Homebrew

```bash
brew tap zircote/tap
brew install rlm-rs
```

### From Source

```bash
git clone https://github.com/zircote/rlm-rs.git
cd rlm-rs
make install
```

## Quick Start

```bash
# Initialize the database
rlm-cli init

# Load a large document (auto-generates embeddings)
rlm-cli load document.md --name docs --chunker semantic

# Search with hybrid semantic + BM25
rlm-cli search "your query" --buffer docs --top-k 10

# Retrieve chunk by ID (pass-by-reference)
rlm-cli chunk get 42

# Check status
rlm-cli status

# Regex search content
rlm-cli grep docs "pattern" --max-matches 20

# View content slice
rlm-cli peek docs --start 0 --end 3000
```

## Commands

| Command | Description |
|---------|-------------|
| `init` | Initialize the RLM database |
| `status` | Show current state (buffers, chunks, DB info) |
| `load` | Load a file into a buffer with chunking (auto-embeds) |
| `search` | Hybrid semantic + BM25 search across chunks |
| `update-buffer` | Update buffer content with re-chunking |
| `dispatch` | Split chunks into batches for parallel subagent processing |
| `aggregate` | Combine findings from analyst subagents |
| `chunk get` | Retrieve chunk by ID (pass-by-reference) |
| `chunk list` | List chunks for a buffer |
| `chunk embed` | Generate embeddings (or re-embed with --force) |
| `chunk status` | Show embedding status |
| `list` | List all buffers |
| `show` | Show buffer details |
| `delete` | Delete a buffer |
| `peek` | View a slice of buffer content |
| `grep` | Search buffer content with regex |
| `write-chunks` | Write chunks to individual files |
| `add-buffer` | Add text to a new buffer |
| `export-buffers` | Export all buffers to JSON |
| `var` | Get/set context variables |
| `global` | Get/set global variables |
| `reset` | Delete all RLM state |

## Chunking Strategies

| Strategy | Best For | Description |
|----------|----------|-------------|
| `semantic` | Markdown, prose | Splits at natural boundaries (headings, paragraphs) |
| `code` | Source code | Language-aware chunking at function/class boundaries |
| `fixed` | Logs, plain text | Splits at exact byte boundaries |
| `parallel` | Large files (>10MB) | Multi-threaded fixed chunking |

```bash
# Semantic chunking (default)
rlm-cli load doc.md --chunker semantic

# Code-aware chunking for source files
rlm-cli load src/main.rs --chunker code

# Fixed chunking with overlap
rlm-cli load logs.txt --chunker fixed --chunk-size 150000 --overlap 1000

# Parallel chunking for speed
rlm-cli load huge.txt --chunker parallel --chunk-size 100000
```

### Supported Languages (Code Chunker)

Rust, Python, JavaScript, TypeScript, Go, Java, C/C++, Ruby, PHP

## Claude Code Integration

rlm-cli is designed to work with the [rlm-rs Claude Code plugin](https://github.com/zircote/rlm-plugin), implementing the RLM architecture:

| RLM Concept | Implementation |
|-------------|----------------|
| Root LLM | Main Claude Code conversation (Opus/Sonnet) |
| Sub-LLM | `rlm-subcall` agent (Haiku) |
| External Environment | `rlm-cli` CLI + SQLite |

## Development

### Prerequisites

- Rust 1.88+ (2024 edition)
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) for supply chain security

### Build

```bash
# Using Makefile
make build          # Debug build
make release        # Release build
make test           # Run tests
make check          # Format + lint + test
make ci             # Full CI check
make install        # Install to ~/.cargo/bin

# Or using Cargo directly
cargo build --release
cargo test
cargo clippy --all-targets --all-features
```

### Project Structure

```
src/
├── lib.rs           # Library entry point
├── main.rs          # CLI entry point
├── error.rs         # Error types
├── core/            # Core types (Buffer, Chunk, Variable)
├── chunking/        # Chunking strategies
├── storage/         # SQLite persistence
├── io/              # File I/O with mmap
└── cli/             # Command implementations

tests/
└── integration_test.rs
```

## MSRV Policy

The Minimum Supported Rust Version (MSRV) is **1.88**.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Documentation

📚 **[Complete Documentation](docs/README.md)** - Full documentation hub with tutorials, guides, and reference materials

### Quick Links

- **[Getting Started](docs/getting-started.md)** - 5-minute tutorial for new users
- **[Examples](docs/examples.md)** - Practical examples and workflows
- **[CLI Reference](docs/cli-reference.md)** - Complete command documentation
- **[FAQ](docs/faq.md)** - Frequently asked questions
- **[Troubleshooting](docs/troubleshooting.md)** - Common issues and solutions
- **[Glossary](docs/glossary.md)** - RLM and chunking terminology

### Advanced Topics

- **[Features Guide](docs/features.md)** - Feature flags and build options
- **[Plugin Integration](docs/plugin-integration.md)** - Integration with Claude Code
- **[Architecture](docs/architecture.md)** - Internal design and architecture
- **[RLM-Inspired Design](docs/rlm-inspired-design.md)** - Connection to RLM paper
- **[API Reference](docs/api.md)** - Rust library documentation
- **[ADRs](docs/adr/)** - Architectural Decision Records

## Citing This Project

If you use rlm-cli in your research or projects, please cite it. You can use GitHub's built-in "Cite this repository" button, or use the following BibTeX:

```bibtex
@software{allen_rlm_cli_2026,
  author       = {Allen, Robert},
  title        = {rlm-cli},
  version      = {1.2.4},
  year         = {2026},
  url          = {https://github.com/zircote/rlm-rs},
  license      = {MIT}
}
```

## Acknowledgments

This project builds on prior work in recursive language model architectures:

- [claude_code_RLM](https://github.com/brainqub3/claude_code_RLM) - Original Python RLM implementation by [Brainqub3](https://brainqub3.com/) that inspired the creation of this project
- [RLM Paper (arXiv:2512.24601)](https://arxiv.org/abs/2512.24601) - Recursive Language Model pattern by Zhang, Kraska, and Khattab (MIT CSAIL)
- [Claude Code](https://claude.ai/code) - AI-powered development environment

> Adeojo, John. *claude_code_RLM*. GitHub, 2026. https://github.com/brainqub3/claude_code_RLM

> Zhang, Alex L., Tim Kraska, and Omar Khattab. "Recursive Language Models." arXiv:2512.24601, 2025. MIT CSAIL. https://arxiv.org/abs/2512.24601
