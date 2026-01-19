//! Binary entry point for RLM-RS.
//!
//! RLM-RS: Recursive Language Model REPL for Claude Code.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use clap::Parser;
use rlm_rs::cli::{Cli, execute};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match execute(&cli) {
        Ok(output) => {
            if !output.is_empty() {
                print!("{output}");
            }
            ExitCode::SUCCESS
        },
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        },
    }
}
