//! Output formatting for CLI commands.
//!
//! Supports text and JSON output formats.

use crate::core::{Buffer, Chunk, Context};
use crate::storage::traits::StorageStats;
use serde::Serialize;
use std::fmt::Write;

/// Output format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable text output.
    Text,
    /// JSON output.
    Json,
}

impl OutputFormat {
    /// Parses format from string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => Self::Json,
            _ => Self::Text,
        }
    }
}

/// Formats a status response.
#[must_use]
pub fn format_status(stats: &StorageStats, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => format_status_text(stats),
        OutputFormat::Json => format_json(stats),
    }
}

fn format_status_text(stats: &StorageStats) -> String {
    let mut output = String::new();
    output.push_str("RLM-RS Status\n");
    output.push_str("=============\n\n");
    let _ = writeln!(output, "  Buffers:       {}", stats.buffer_count);
    let _ = writeln!(output, "  Chunks:        {}", stats.chunk_count);
    let _ = writeln!(
        output,
        "  Content size:  {} bytes",
        stats.total_content_size
    );
    let _ = writeln!(
        output,
        "  Context:       {}",
        if stats.has_context { "yes" } else { "no" }
    );
    let _ = writeln!(output, "  Schema:        v{}", stats.schema_version);
    if let Some(size) = stats.db_size {
        let _ = writeln!(output, "  DB size:       {size} bytes");
    }
    output
}

/// Formats a buffer list.
#[must_use]
pub fn format_buffer_list(buffers: &[Buffer], format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => format_buffer_list_text(buffers),
        OutputFormat::Json => format_json(&buffers),
    }
}

fn format_buffer_list_text(buffers: &[Buffer]) -> String {
    if buffers.is_empty() {
        return "No buffers found.\n".to_string();
    }

    let mut output = String::new();
    output.push_str("Buffers:\n");
    let _ = writeln!(
        output,
        "{:<6} {:<20} {:<12} {:<8} Source",
        "ID", "Name", "Size", "Chunks"
    );
    output.push_str(&"-".repeat(70));
    output.push('\n');

    for buffer in buffers {
        let id = buffer.id.map_or_else(|| "-".to_string(), |i| i.to_string());
        let name = buffer.name.as_deref().unwrap_or("-");
        let size = format_size(buffer.metadata.size);
        let chunks = buffer
            .metadata
            .chunk_count
            .map_or_else(|| "-".to_string(), |c| c.to_string());
        let source = buffer
            .source
            .as_ref()
            .map_or_else(|| "-".to_string(), |p| p.to_string_lossy().to_string());

        let _ = writeln!(
            output,
            "{:<6} {:<20} {:<12} {:<8} {}",
            id,
            truncate(name, 20),
            size,
            chunks,
            truncate(&source, 30)
        );
    }

    output
}

/// Formats a single buffer.
#[must_use]
pub fn format_buffer(buffer: &Buffer, chunks: Option<&[Chunk]>, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => format_buffer_text(buffer, chunks),
        OutputFormat::Json => {
            #[derive(Serialize)]
            struct BufferWithChunks<'a> {
                buffer: &'a Buffer,
                chunks: Option<&'a [Chunk]>,
            }
            format_json(&BufferWithChunks { buffer, chunks })
        }
    }
}

fn format_buffer_text(buffer: &Buffer, chunks: Option<&[Chunk]>) -> String {
    let mut output = String::new();

    let _ = writeln!(
        output,
        "Buffer: {}",
        buffer.name.as_deref().unwrap_or("unnamed")
    );
    let _ = writeln!(output, "  ID:           {}", buffer.id.unwrap_or(0));
    let _ = writeln!(output, "  Size:         {} bytes", buffer.metadata.size);
    if let Some(lines) = buffer.metadata.line_count {
        let _ = writeln!(output, "  Lines:        {lines}");
    }
    if let Some(chunk_count) = buffer.metadata.chunk_count {
        let _ = writeln!(output, "  Chunks:       {chunk_count}");
    }
    if let Some(ref ct) = buffer.metadata.content_type {
        let _ = writeln!(output, "  Content type: {ct}");
    }
    if let Some(ref source) = buffer.source {
        let _ = writeln!(output, "  Source:       {}", source.display());
    }

    if let Some(chunks) = chunks {
        output.push('\n');
        output.push_str("Chunks:\n");
        let _ = writeln!(
            output,
            "{:<6} {:<12} {:<12} {:<10} Preview",
            "Index", "Start", "End", "Size"
        );
        output.push_str(&"-".repeat(70));
        output.push('\n');

        for chunk in chunks {
            let preview = truncate(&chunk.content.replace('\n', "\\n"), 30);
            let _ = writeln!(
                output,
                "{:<6} {:<12} {:<12} {:<10} {}",
                chunk.index,
                chunk.byte_range.start,
                chunk.byte_range.end,
                chunk.size(),
                preview
            );
        }
    }

    output
}

/// Formats peek output.
#[must_use]
pub fn format_peek(content: &str, start: usize, end: usize, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => {
            let mut output = String::new();
            let _ = writeln!(output, "Bytes {start}..{end} ({} bytes):", end - start);
            output.push_str("---\n");
            output.push_str(content);
            if !content.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("---\n");
            output
        }
        OutputFormat::Json => {
            #[derive(Serialize)]
            struct PeekOutput<'a> {
                start: usize,
                end: usize,
                size: usize,
                content: &'a str,
            }
            format_json(&PeekOutput {
                start,
                end,
                size: end - start,
                content,
            })
        }
    }
}

/// Formats grep matches.
#[must_use]
pub fn format_grep_matches(matches: &[GrepMatch], pattern: &str, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => format_grep_text(matches, pattern),
        OutputFormat::Json => format_json(&matches),
    }
}

fn format_grep_text(matches: &[GrepMatch], pattern: &str) -> String {
    if matches.is_empty() {
        return format!("No matches found for pattern: {pattern}\n");
    }

    let mut output = String::new();
    let _ = writeln!(
        output,
        "Found {} matches for pattern: {pattern}\n",
        matches.len()
    );

    for (i, m) in matches.iter().enumerate() {
        let _ = writeln!(output, "Match {} at byte {}:", i + 1, m.offset);
        let _ = writeln!(output, "  {}", m.snippet.replace('\n', "\\n"));
    }

    output
}

/// Formats chunk indices.
#[must_use]
pub fn format_chunk_indices(indices: &[(usize, usize)], format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => {
            let mut output = String::new();
            let _ = writeln!(output, "{} chunks:", indices.len());
            for (i, (start, end)) in indices.iter().enumerate() {
                let _ = writeln!(output, "  [{i}] {start}..{end} ({} bytes)", end - start);
            }
            output
        }
        OutputFormat::Json => format_json(&indices),
    }
}

/// Formats write chunks result.
#[must_use]
pub fn format_write_chunks_result(paths: &[String], format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => {
            let mut output = String::new();
            let _ = writeln!(output, "Wrote {} chunks:", paths.len());
            for path in paths {
                let _ = writeln!(output, "  {path}");
            }
            output
        }
        OutputFormat::Json => format_json(&paths),
    }
}

/// Formats context.
#[must_use]
pub fn format_context(context: &Context, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => {
            let mut output = String::new();
            output.push_str("Context:\n");
            let _ = writeln!(output, "  Variables: {}", context.variable_count());
            let _ = writeln!(output, "  Globals:   {}", context.global_count());
            let _ = writeln!(output, "  Buffers:   {}", context.buffer_count());
            output
        }
        OutputFormat::Json => format_json(&context),
    }
}

/// A grep match result.
#[derive(Debug, Clone, Serialize)]
pub struct GrepMatch {
    /// Byte offset in the buffer.
    pub offset: usize,
    /// The matched text.
    pub matched: String,
    /// Context snippet around the match.
    pub snippet: String,
}

/// Formats a value as JSON.
fn format_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
}

/// Formats a byte size as human-readable.
#[allow(clippy::cast_precision_loss)]
fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Truncates a string to max length with ellipsis.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s[..max_len].to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(OutputFormat::parse("json"), OutputFormat::Json);
        assert_eq!(OutputFormat::parse("JSON"), OutputFormat::Json);
        assert_eq!(OutputFormat::parse("text"), OutputFormat::Text);
        assert_eq!(OutputFormat::parse("unknown"), OutputFormat::Text);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(100), "100 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Hello", 10), "Hello");
        assert_eq!(truncate("Hello World", 8), "Hello...");
        assert_eq!(truncate("Hi", 2), "Hi");
    }

    #[test]
    fn test_format_status() {
        let stats = StorageStats {
            buffer_count: 2,
            chunk_count: 10,
            total_content_size: 1024,
            has_context: true,
            schema_version: 1,
            db_size: Some(4096),
        };

        let text = format_status(&stats, OutputFormat::Text);
        assert!(text.contains("Buffers:       2"));
        assert!(text.contains("Chunks:        10"));

        let json = format_status(&stats, OutputFormat::Json);
        assert!(json.contains("\"buffer_count\": 2"));
    }
}
