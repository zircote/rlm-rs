//! Command-line argument parsing.
//!
//! Defines the CLI structure using clap derive macros.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// RLM-RS: Recursive Language Model REPL for Claude Code.
///
/// A CLI tool for handling large context files via chunking and
/// recursive sub-LLM calls.
#[derive(Parser, Debug)]
#[command(name = "rlm-rs")]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Path to the RLM database file.
    ///
    /// Defaults to `.rlm/rlm-state.db` in the current directory.
    #[arg(short, long, env = "RLM_DB_PATH")]
    pub db_path: Option<PathBuf>,

    /// Enable verbose output.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output format (text, json).
    #[arg(long, default_value = "text", global = true)]
    pub format: String,

    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize the RLM database.
    ///
    /// Creates the database file and schema if they don't exist.
    Init {
        /// Force re-initialization (destroys existing data).
        #[arg(short, long)]
        force: bool,
    },

    /// Show current RLM state status.
    Status,

    /// Reset RLM state (delete all data).
    Reset {
        /// Skip confirmation prompt.
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Load a context file into a buffer.
    Load {
        /// Path to the context file.
        file: PathBuf,

        /// Optional name for the buffer.
        #[arg(short, long)]
        name: Option<String>,

        /// Chunking strategy (fixed, semantic, parallel).
        #[arg(short, long, default_value = "semantic")]
        chunker: String,

        /// Chunk size in characters (~10k tokens at 4 chars/token).
        #[arg(long, default_value = "40000")]
        chunk_size: usize,

        /// Overlap between chunks in characters.
        #[arg(long, default_value = "500")]
        overlap: usize,
    },

    /// List all buffers.
    #[command(name = "list", alias = "ls")]
    ListBuffers,

    /// Show buffer details.
    #[command(name = "show")]
    ShowBuffer {
        /// Buffer ID or name.
        buffer: String,

        /// Show chunks as well.
        #[arg(short, long)]
        chunks: bool,
    },

    /// Delete a buffer.
    #[command(name = "delete", alias = "rm")]
    DeleteBuffer {
        /// Buffer ID or name.
        buffer: String,

        /// Skip confirmation prompt.
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Peek at buffer content.
    Peek {
        /// Buffer ID or name.
        buffer: String,

        /// Start offset in bytes.
        #[arg(long, default_value = "0")]
        start: usize,

        /// End offset in bytes (default: start + 3000).
        #[arg(long)]
        end: Option<usize>,
    },

    /// Search buffer content with regex.
    Grep {
        /// Buffer ID or name.
        buffer: String,

        /// Search pattern (regex).
        pattern: String,

        /// Maximum number of matches.
        #[arg(short = 'n', long, default_value = "20")]
        max_matches: usize,

        /// Context window size around matches.
        #[arg(short, long, default_value = "120")]
        window: usize,

        /// Case-insensitive search.
        #[arg(short, long)]
        ignore_case: bool,
    },

    /// Get chunk indices for a buffer.
    ChunkIndices {
        /// Buffer ID or name.
        buffer: String,

        /// Chunk size in characters (~10k tokens at 4 chars/token).
        #[arg(long, default_value = "40000")]
        chunk_size: usize,

        /// Overlap between chunks in characters.
        #[arg(long, default_value = "500")]
        overlap: usize,
    },

    /// Write chunks to files.
    WriteChunks {
        /// Buffer ID or name.
        buffer: String,

        /// Output directory.
        #[arg(short, long, default_value = ".rlm/chunks")]
        out_dir: PathBuf,

        /// Chunk size in characters (~10k tokens at 4 chars/token).
        #[arg(long, default_value = "40000")]
        chunk_size: usize,

        /// Overlap between chunks in characters.
        #[arg(long, default_value = "500")]
        overlap: usize,

        /// Filename prefix.
        #[arg(long, default_value = "chunk")]
        prefix: String,
    },

    /// Add text to a buffer (intermediate results).
    AddBuffer {
        /// Buffer name.
        name: String,

        /// Content to add (reads from stdin if not provided).
        content: Option<String>,
    },

    /// Export all buffers to a file.
    ExportBuffers {
        /// Output file path (stdout if not specified).
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Pretty-print if JSON format.
        #[arg(short, long)]
        pretty: bool,
    },

    /// Set or get context variables.
    #[command(name = "var")]
    Variable {
        /// Variable name.
        name: String,

        /// Value to set (omit to get current value).
        value: Option<String>,

        /// Delete the variable.
        #[arg(short, long)]
        delete: bool,
    },

    /// Set or get global variables.
    Global {
        /// Variable name.
        name: String,

        /// Value to set (omit to get current value).
        value: Option<String>,

        /// Delete the variable.
        #[arg(short, long)]
        delete: bool,
    },
}

impl Cli {
    /// Returns the database path, using the default if not specified.
    #[must_use]
    pub fn get_db_path(&self) -> PathBuf {
        self.db_path
            .clone()
            .unwrap_or_else(|| PathBuf::from(crate::storage::DEFAULT_DB_PATH))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parse() {
        // Test that CLI can be created
        Cli::command().debug_assert();
    }

    #[test]
    fn test_default_db_path() {
        let cli = Cli {
            db_path: None,
            verbose: false,
            format: "text".to_string(),
            command: Commands::Status,
        };
        assert_eq!(
            cli.get_db_path(),
            PathBuf::from(crate::storage::DEFAULT_DB_PATH)
        );
    }

    #[test]
    fn test_custom_db_path() {
        let cli = Cli {
            db_path: Some(PathBuf::from("/custom/path.db")),
            verbose: false,
            format: "text".to_string(),
            command: Commands::Status,
        };
        assert_eq!(cli.get_db_path(), PathBuf::from("/custom/path.db"));
    }
}
