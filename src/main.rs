//! Binary entry point for RLM-RS.
//!
//! RLM-RS: Recursive Language Model REPL for Claude Code.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use clap::Parser;
use rlm_rs::cli::output::{OutputFormat, format_error};
use rlm_rs::cli::{Cli, execute};
use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let format = OutputFormat::parse(&cli.format);

    match execute(&cli) {
        Ok(output) => {
            if !output.is_empty() {
                // Handle broken pipe gracefully (e.g., when piped to `head` or `jq`)
                if let Err(e) = write!(io::stdout(), "{output}")
                    && e.kind() != io::ErrorKind::BrokenPipe
                {
                    eprintln!("Error writing to stdout: {e}");
                    return ExitCode::FAILURE;
                }
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            let error_output = format_error(&e, format);
            match format {
                OutputFormat::Json | OutputFormat::Ndjson => {
                    // JSON errors go to stdout for programmatic parsing
                    println!("{error_output}");
                }
                OutputFormat::Text => {
                    eprintln!("Error: {error_output}");
                }
            }
            ExitCode::FAILURE
        }
    }
}
