//! CLI command implementations.
//!
//! Contains the business logic for each CLI command.

use crate::chunking::{ChunkerMetadata, create_chunker};
use crate::cli::output::{
    GrepMatch, OutputFormat, format_buffer, format_buffer_list, format_chunk_indices,
    format_grep_matches, format_peek, format_status, format_write_chunks_result,
};
use crate::cli::parser::{Cli, Commands};
use crate::core::{Buffer, Context, ContextValue};
use crate::error::{CommandError, Result, StorageError};
use crate::io::{read_file, write_file};
use crate::storage::{SqliteStorage, Storage};
use regex::RegexBuilder;
use std::io::{self, Read, Write as IoWrite};

/// Executes the CLI command.
///
/// # Arguments
///
/// * `cli` - Parsed CLI arguments.
///
/// # Returns
///
/// Result with output string on success.
///
/// # Errors
///
/// Returns an error if the command fails to execute.
pub fn execute(cli: &Cli) -> Result<String> {
    let format = OutputFormat::parse(&cli.format);
    let db_path = cli.get_db_path();

    match &cli.command {
        Commands::Init { force } => cmd_init(&db_path, *force, format),
        Commands::Status => cmd_status(&db_path, format),
        Commands::Reset { yes } => cmd_reset(&db_path, *yes, format),
        Commands::Load {
            file,
            name,
            chunker,
            chunk_size,
            overlap,
        } => cmd_load(
            &db_path,
            file,
            name.as_deref(),
            chunker,
            *chunk_size,
            *overlap,
            format,
        ),
        Commands::ListBuffers => cmd_list_buffers(&db_path, format),
        Commands::ShowBuffer { buffer, chunks } => {
            cmd_show_buffer(&db_path, buffer, *chunks, format)
        },
        Commands::DeleteBuffer { buffer, yes } => cmd_delete_buffer(&db_path, buffer, *yes, format),
        Commands::Peek { buffer, start, end } => cmd_peek(&db_path, buffer, *start, *end, format),
        Commands::Grep {
            buffer,
            pattern,
            max_matches,
            window,
            ignore_case,
        } => cmd_grep(
            &db_path,
            buffer,
            pattern,
            *max_matches,
            *window,
            *ignore_case,
            format,
        ),
        Commands::ChunkIndices {
            buffer,
            chunk_size,
            overlap,
        } => cmd_chunk_indices(&db_path, buffer, *chunk_size, *overlap, format),
        Commands::WriteChunks {
            buffer,
            out_dir,
            chunk_size,
            overlap,
            prefix,
        } => cmd_write_chunks(
            &db_path,
            buffer,
            out_dir,
            *chunk_size,
            *overlap,
            prefix,
            format,
        ),
        Commands::AddBuffer { name, content } => {
            cmd_add_buffer(&db_path, name, content.as_deref(), format)
        },
        Commands::ExportBuffers { output, pretty } => {
            cmd_export_buffers(&db_path, output.as_deref(), *pretty, format)
        },
        Commands::Variable {
            name,
            value,
            delete,
        } => cmd_variable(&db_path, name, value.as_deref(), *delete, format),
        Commands::Global {
            name,
            value,
            delete,
        } => cmd_global(&db_path, name, value.as_deref(), *delete, format),
    }
}

/// Opens storage and ensures it's initialized.
fn open_storage(db_path: &std::path::Path) -> Result<SqliteStorage> {
    let storage = SqliteStorage::open(db_path)?;

    if !storage.is_initialized()? {
        return Err(StorageError::NotInitialized.into());
    }

    Ok(storage)
}

/// Resolves a buffer identifier (ID or name) to a buffer.
fn resolve_buffer(storage: &SqliteStorage, identifier: &str) -> Result<Buffer> {
    // Try as ID first
    if let Ok(id) = identifier.parse::<i64>() {
        if let Some(buffer) = storage.get_buffer(id)? {
            return Ok(buffer);
        }
    }

    // Try as name
    if let Some(buffer) = storage.get_buffer_by_name(identifier)? {
        return Ok(buffer);
    }

    Err(StorageError::BufferNotFound {
        identifier: identifier.to_string(),
    }
    .into())
}

// ==================== Command Implementations ====================

fn cmd_init(db_path: &std::path::Path, force: bool, _format: OutputFormat) -> Result<String> {
    // Check if already exists
    if db_path.exists() && !force {
        return Err(CommandError::ExecutionFailed(
            "Database already exists. Use --force to reinitialize.".to_string(),
        )
        .into());
    }

    // Create parent directory if needed
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                CommandError::ExecutionFailed(format!("Failed to create directory: {e}"))
            })?;
        }
    }

    // If force, delete existing
    if force && db_path.exists() {
        std::fs::remove_file(db_path).map_err(|e| {
            CommandError::ExecutionFailed(format!("Failed to remove existing database: {e}"))
        })?;
    }

    let mut storage = SqliteStorage::open(db_path)?;
    storage.init()?;

    // Initialize empty context
    let context = Context::new();
    storage.save_context(&context)?;

    Ok(format!(
        "Initialized RLM database at: {}\n",
        db_path.display()
    ))
}

fn cmd_status(db_path: &std::path::Path, format: OutputFormat) -> Result<String> {
    let storage = open_storage(db_path)?;
    let stats = storage.stats()?;
    Ok(format_status(&stats, format))
}

fn cmd_reset(db_path: &std::path::Path, yes: bool, _format: OutputFormat) -> Result<String> {
    if !yes {
        // In a real implementation, we'd prompt the user
        // For now, require --yes flag
        return Err(CommandError::ExecutionFailed(
            "Use --yes to confirm reset. This will delete all data.".to_string(),
        )
        .into());
    }

    let mut storage = open_storage(db_path)?;
    storage.reset()?;

    // Reinitialize with empty context
    let context = Context::new();
    storage.save_context(&context)?;

    Ok("RLM state reset successfully.\n".to_string())
}

fn cmd_load(
    db_path: &std::path::Path,
    file: &std::path::Path,
    name: Option<&str>,
    chunker_name: &str,
    chunk_size: usize,
    overlap: usize,
    format: OutputFormat,
) -> Result<String> {
    let mut storage = open_storage(db_path)?;

    // Read file content
    let content = read_file(file)?;

    // Create buffer
    let buffer_name = name
        .map(String::from)
        .or_else(|| file.file_name().and_then(|n| n.to_str()).map(String::from));

    let mut buffer = Buffer::from_file(file.to_path_buf(), content.clone());
    buffer.name = buffer_name;
    buffer.compute_hash();

    // Add buffer to storage
    let buffer_id = storage.add_buffer(&buffer)?;

    // Chunk the content
    let chunker = create_chunker(chunker_name)?;
    let meta = ChunkerMetadata::with_size_and_overlap(chunk_size, overlap);
    let chunks = chunker.chunk(buffer_id, &content, Some(&meta))?;

    // Store chunks
    storage.add_chunks(buffer_id, &chunks)?;

    // Update buffer with chunk count
    let mut updated_buffer =
        storage
            .get_buffer(buffer_id)?
            .ok_or_else(|| StorageError::BufferNotFound {
                identifier: buffer_id.to_string(),
            })?;
    updated_buffer.set_chunk_count(chunks.len());
    storage.update_buffer(&updated_buffer)?;

    // Update context
    if let Some(mut context) = storage.load_context()? {
        context.add_buffer(buffer_id);
        storage.save_context(&context)?;
    }

    match format {
        OutputFormat::Text => Ok(format!(
            "Loaded buffer {} (ID: {}) with {} chunks from {}\n",
            updated_buffer.name.as_deref().unwrap_or("unnamed"),
            buffer_id,
            chunks.len(),
            file.display()
        )),
        OutputFormat::Json => {
            let result = serde_json::json!({
                "buffer_id": buffer_id,
                "name": updated_buffer.name,
                "chunk_count": chunks.len(),
                "size": content.len(),
                "source": file.to_string_lossy()
            });
            Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
        },
    }
}

fn cmd_list_buffers(db_path: &std::path::Path, format: OutputFormat) -> Result<String> {
    let storage = open_storage(db_path)?;
    let buffers = storage.list_buffers()?;
    Ok(format_buffer_list(&buffers, format))
}

fn cmd_show_buffer(
    db_path: &std::path::Path,
    identifier: &str,
    show_chunks: bool,
    format: OutputFormat,
) -> Result<String> {
    let storage = open_storage(db_path)?;
    let buffer = resolve_buffer(&storage, identifier)?;

    let chunks = if show_chunks {
        Some(storage.get_chunks(buffer.id.unwrap_or(0))?)
    } else {
        None
    };

    Ok(format_buffer(&buffer, chunks.as_deref(), format))
}

fn cmd_delete_buffer(
    db_path: &std::path::Path,
    identifier: &str,
    yes: bool,
    _format: OutputFormat,
) -> Result<String> {
    if !yes {
        return Err(
            CommandError::ExecutionFailed("Use --yes to confirm deletion.".to_string()).into(),
        );
    }

    let mut storage = open_storage(db_path)?;
    let buffer = resolve_buffer(&storage, identifier)?;
    let buffer_id = buffer.id.unwrap_or(0);
    let buffer_name = buffer.name.unwrap_or_else(|| format!("{buffer_id}"));

    storage.delete_buffer(buffer_id)?;

    // Update context
    if let Some(mut context) = storage.load_context()? {
        context.remove_buffer(buffer_id);
        storage.save_context(&context)?;
    }

    Ok(format!("Deleted buffer: {buffer_name}\n"))
}

fn cmd_peek(
    db_path: &std::path::Path,
    identifier: &str,
    start: usize,
    end: Option<usize>,
    format: OutputFormat,
) -> Result<String> {
    let storage = open_storage(db_path)?;
    let buffer = resolve_buffer(&storage, identifier)?;

    let end = end.unwrap_or(start + 3000).min(buffer.content.len());
    let start = start.min(buffer.content.len());

    let content = buffer.slice(start, end).unwrap_or("");
    Ok(format_peek(content, start, end, format))
}

fn cmd_grep(
    db_path: &std::path::Path,
    identifier: &str,
    pattern: &str,
    max_matches: usize,
    window: usize,
    ignore_case: bool,
    format: OutputFormat,
) -> Result<String> {
    let storage = open_storage(db_path)?;
    let buffer = resolve_buffer(&storage, identifier)?;

    let regex = RegexBuilder::new(pattern)
        .case_insensitive(ignore_case)
        .build()
        .map_err(|e| CommandError::InvalidArgument(format!("Invalid regex: {e}")))?;

    let mut matches = Vec::new();
    for m in regex.find_iter(&buffer.content) {
        if matches.len() >= max_matches {
            break;
        }

        let start = m.start().saturating_sub(window);
        let end = (m.end() + window).min(buffer.content.len());

        // Find valid UTF-8 boundaries
        let start = crate::io::find_char_boundary(&buffer.content, start);
        let end = crate::io::find_char_boundary(&buffer.content, end);

        matches.push(GrepMatch {
            offset: m.start(),
            matched: m.as_str().to_string(),
            snippet: buffer.content[start..end].to_string(),
        });
    }

    Ok(format_grep_matches(&matches, pattern, format))
}

fn cmd_chunk_indices(
    db_path: &std::path::Path,
    identifier: &str,
    chunk_size: usize,
    overlap: usize,
    format: OutputFormat,
) -> Result<String> {
    let storage = open_storage(db_path)?;
    let buffer = resolve_buffer(&storage, identifier)?;

    let content_len = buffer.content.len();
    let mut indices = Vec::new();

    if chunk_size == 0 || overlap >= chunk_size {
        return Err(
            CommandError::InvalidArgument("Invalid chunk_size or overlap".to_string()).into(),
        );
    }

    let step = chunk_size - overlap;
    let mut start = 0;

    while start < content_len {
        let end = (start + chunk_size).min(content_len);
        indices.push((start, end));
        if end >= content_len {
            break;
        }
        start += step;
    }

    Ok(format_chunk_indices(&indices, format))
}

fn cmd_write_chunks(
    db_path: &std::path::Path,
    identifier: &str,
    out_dir: &std::path::Path,
    chunk_size: usize,
    overlap: usize,
    prefix: &str,
    format: OutputFormat,
) -> Result<String> {
    let mut storage = open_storage(db_path)?;
    let buffer = resolve_buffer(&storage, identifier)?;
    let buffer_id = buffer.id.unwrap_or(0);

    // Create chunker and chunk the content
    let chunker = create_chunker("semantic")?;
    let meta = ChunkerMetadata::with_size_and_overlap(chunk_size, overlap);
    let chunks = chunker.chunk(buffer_id, &buffer.content, Some(&meta))?;

    // Store chunks in SQLite
    storage.add_chunks(buffer_id, &chunks)?;

    // Update buffer with chunk count
    let mut updated_buffer =
        storage
            .get_buffer(buffer_id)?
            .ok_or_else(|| StorageError::BufferNotFound {
                identifier: buffer_id.to_string(),
            })?;
    updated_buffer.set_chunk_count(chunks.len());
    storage.update_buffer(&updated_buffer)?;

    // Write chunks to files
    let chunks_iter = chunks
        .iter()
        .enumerate()
        .map(|(i, c)| (i, c.content.as_str()));
    let paths = crate::io::reader::write_chunks(out_dir, chunks_iter, prefix)?;

    Ok(format_write_chunks_result(&paths, format))
}

fn cmd_add_buffer(
    db_path: &std::path::Path,
    name: &str,
    content: Option<&str>,
    format: OutputFormat,
) -> Result<String> {
    let mut storage = open_storage(db_path)?;

    // Read content from stdin if not provided
    let content = if let Some(c) = content {
        c.to_string()
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).map_err(|e| {
            CommandError::ExecutionFailed(format!("Failed to read from stdin: {e}"))
        })?;
        buffer
    };

    let buffer = Buffer::from_named(name.to_string(), content.clone());
    let buffer_id = storage.add_buffer(&buffer)?;

    // Update context
    if let Some(mut context) = storage.load_context()? {
        context.add_buffer(buffer_id);
        storage.save_context(&context)?;
    }

    match format {
        OutputFormat::Text => Ok(format!(
            "Added buffer '{}' (ID: {}, {} bytes)\n",
            name,
            buffer_id,
            content.len()
        )),
        OutputFormat::Json => {
            let result = serde_json::json!({
                "buffer_id": buffer_id,
                "name": name,
                "size": content.len()
            });
            Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
        },
    }
}

fn cmd_export_buffers(
    db_path: &std::path::Path,
    output: Option<&std::path::Path>,
    _pretty: bool,
    _format: OutputFormat,
) -> Result<String> {
    let storage = open_storage(db_path)?;
    let content = storage.export_buffers()?;

    if let Some(path) = output {
        write_file(path, &content)?;
        Ok(format!("Exported buffers to: {}\n", path.display()))
    } else {
        // Write to stdout
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(content.as_bytes()).map_err(|e| {
            CommandError::ExecutionFailed(format!("Failed to write to stdout: {e}"))
        })?;
        Ok(String::new()) // Content already written
    }
}

fn cmd_variable(
    db_path: &std::path::Path,
    name: &str,
    value: Option<&str>,
    delete: bool,
    format: OutputFormat,
) -> Result<String> {
    let mut storage = open_storage(db_path)?;
    let mut context = storage.load_context()?.unwrap_or_else(Context::new);

    if delete {
        context.remove_variable(name);
        storage.save_context(&context)?;
        return Ok(format!("Deleted variable: {name}\n"));
    }

    if let Some(v) = value {
        context.set_variable(name.to_string(), ContextValue::String(v.to_string()));
        storage.save_context(&context)?;
        Ok(format!("Set variable: {name} = {v}\n"))
    } else {
        context.get_variable(name).map_or_else(
            || Ok(format!("Variable '{name}' not found\n")),
            |v| match format {
                OutputFormat::Text => Ok(format!("{name} = {v:?}\n")),
                OutputFormat::Json => Ok(serde_json::to_string_pretty(v).unwrap_or_default()),
            },
        )
    }
}

fn cmd_global(
    db_path: &std::path::Path,
    name: &str,
    value: Option<&str>,
    delete: bool,
    format: OutputFormat,
) -> Result<String> {
    let mut storage = open_storage(db_path)?;
    let mut context = storage.load_context()?.unwrap_or_else(Context::new);

    if delete {
        context.remove_global(name);
        storage.save_context(&context)?;
        return Ok(format!("Deleted global: {name}\n"));
    }

    if let Some(v) = value {
        context.set_global(name.to_string(), ContextValue::String(v.to_string()));
        storage.save_context(&context)?;
        Ok(format!("Set global: {name} = {v}\n"))
    } else {
        context.get_global(name).map_or_else(
            || Ok(format!("Global '{name}' not found\n")),
            |v| match format {
                OutputFormat::Text => Ok(format!("{name} = {v:?}\n")),
                OutputFormat::Json => Ok(serde_json::to_string_pretty(v).unwrap_or_default()),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        (temp_dir, db_path)
    }

    #[test]
    fn test_cmd_init() {
        let (_temp_dir, db_path) = setup();
        let result = cmd_init(&db_path, false, OutputFormat::Text);
        assert!(result.is_ok());
        assert!(db_path.exists());
    }

    #[test]
    fn test_cmd_init_already_exists() {
        let (_temp_dir, db_path) = setup();

        // First init
        cmd_init(&db_path, false, OutputFormat::Text).unwrap();

        // Second init should fail without force
        let result = cmd_init(&db_path, false, OutputFormat::Text);
        assert!(result.is_err());

        // With force should succeed
        let result = cmd_init(&db_path, true, OutputFormat::Text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_status() {
        let (_temp_dir, db_path) = setup();
        cmd_init(&db_path, false, OutputFormat::Text).unwrap();

        let result = cmd_status(&db_path, OutputFormat::Text);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Buffers"));
    }

    #[test]
    fn test_cmd_reset() {
        let (_temp_dir, db_path) = setup();
        cmd_init(&db_path, false, OutputFormat::Text).unwrap();

        // Without --yes should fail
        let result = cmd_reset(&db_path, false, OutputFormat::Text);
        assert!(result.is_err());

        // With --yes should succeed
        let result = cmd_reset(&db_path, true, OutputFormat::Text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_add_buffer() {
        let (_temp_dir, db_path) = setup();
        cmd_init(&db_path, false, OutputFormat::Text).unwrap();

        let result = cmd_add_buffer(
            &db_path,
            "test-buffer",
            Some("Hello, world!"),
            OutputFormat::Text,
        );
        assert!(result.is_ok());
        assert!(result.unwrap().contains("test-buffer"));
    }

    #[test]
    fn test_cmd_list_buffers() {
        let (_temp_dir, db_path) = setup();
        cmd_init(&db_path, false, OutputFormat::Text).unwrap();

        // Empty list
        let result = cmd_list_buffers(&db_path, OutputFormat::Text);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("No buffers"));

        // Add a buffer
        cmd_add_buffer(&db_path, "test", Some("content"), OutputFormat::Text).unwrap();

        let result = cmd_list_buffers(&db_path, OutputFormat::Text);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("test"));
    }

    #[test]
    fn test_cmd_variable() {
        let (_temp_dir, db_path) = setup();
        cmd_init(&db_path, false, OutputFormat::Text).unwrap();

        // Set variable
        let result = cmd_variable(&db_path, "key", Some("value"), false, OutputFormat::Text);
        assert!(result.is_ok());

        // Get variable
        let result = cmd_variable(&db_path, "key", None, false, OutputFormat::Text);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("value"));

        // Delete variable
        let result = cmd_variable(&db_path, "key", None, true, OutputFormat::Text);
        assert!(result.is_ok());
    }
}
