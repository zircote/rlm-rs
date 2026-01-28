//! Code-aware chunking strategy.
//!
//! Chunks source code at natural boundaries (functions, classes, methods)
//! using regex-based pattern matching for multiple languages.

use crate::chunking::traits::{ChunkMetadata, Chunker};
use crate::chunking::{DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP};
use crate::core::Chunk;
use crate::error::Result;
use regex::Regex;
use std::ops::Range;
use std::sync::OnceLock;

/// Code-aware chunker that splits at function/class boundaries.
///
/// Supports multiple programming languages and falls back to
/// line-based chunking for unknown languages.
///
/// # Supported Languages
///
/// - Rust (.rs)
/// - Python (.py)
/// - JavaScript (.js, .jsx)
/// - TypeScript (.ts, .tsx)
/// - Go (.go)
/// - Java (.java)
/// - C/C++ (.c, .cpp, .h, .hpp)
/// - Ruby (.rb)
/// - PHP (.php)
///
/// # Examples
///
/// ```
/// use rlm_rs::chunking::{Chunker, CodeChunker, ChunkerMetadata};
///
/// let chunker = CodeChunker::new();
/// let code = r#"
/// fn main() {
///     println!("Hello");
/// }
///
/// fn helper() {
///     println!("Helper");
/// }
/// "#;
///
/// let meta = ChunkerMetadata::new().content_type("rs");
/// let chunks = chunker.chunk(1, code, Some(&meta)).unwrap();
/// assert!(!chunks.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct CodeChunker {
    /// Target chunk size in characters.
    chunk_size: usize,
    /// Overlap between consecutive chunks.
    overlap: usize,
}

impl Default for CodeChunker {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeChunker {
    /// Creates a new code chunker with default settings.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            overlap: DEFAULT_OVERLAP,
        }
    }

    /// Creates a code chunker with custom chunk size.
    #[must_use]
    pub const fn with_size(chunk_size: usize) -> Self {
        Self {
            chunk_size,
            overlap: DEFAULT_OVERLAP,
        }
    }

    /// Creates a code chunker with custom size and overlap.
    #[must_use]
    pub const fn with_size_and_overlap(chunk_size: usize, overlap: usize) -> Self {
        Self {
            chunk_size,
            overlap,
        }
    }

    /// Detects language from file extension or content type.
    fn detect_language(metadata: Option<&ChunkMetadata>) -> Language {
        let ext = metadata
            .and_then(|m| {
                m.content_type
                    .as_deref()
                    .or_else(|| m.source.as_deref().and_then(|s| s.rsplit('.').next()))
            })
            .unwrap_or("");

        Language::from_extension(ext)
    }

    /// Finds code structure boundaries in the text.
    #[allow(clippy::unused_self)]
    fn find_boundaries(&self, text: &str, lang: Language) -> Vec<usize> {
        let patterns = lang.boundary_patterns();
        let mut boundaries = Vec::new();

        for pattern in patterns {
            let re = pattern.regex();
            for m in re.find_iter(text) {
                // Find the start of the line containing this match
                let line_start = text[..m.start()].rfind('\n').map_or(0, |pos| pos + 1);
                if !boundaries.contains(&line_start) {
                    boundaries.push(line_start);
                }
            }
        }

        boundaries.sort_unstable();
        boundaries
    }

    /// Chunks text at code boundaries.
    fn chunk_at_boundaries(
        &self,
        buffer_id: i64,
        text: &str,
        boundaries: &[usize],
        chunk_size: usize,
        overlap: usize,
    ) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let mut chunk_start = 0;
        let mut chunk_index = 0;

        while chunk_start < text.len() {
            // Find the end of this chunk
            let ideal_end = (chunk_start + chunk_size).min(text.len());

            // Try to find a boundary near the ideal end
            let chunk_end = self.find_best_boundary(text, chunk_start, ideal_end, boundaries);

            // Extract content
            let content = &text[chunk_start..chunk_end];

            if !content.trim().is_empty() {
                chunks.push(Chunk::new(
                    buffer_id,
                    content.to_string(),
                    Range {
                        start: chunk_start,
                        end: chunk_end,
                    },
                    chunk_index,
                ));
                chunk_index += 1;
            }

            // Move to next chunk with overlap
            if chunk_end >= text.len() {
                break;
            }

            // Calculate next start with overlap
            let next_start = if overlap > 0 {
                self.find_overlap_start(text, chunk_end, overlap, boundaries)
            } else {
                chunk_end
            };

            chunk_start = next_start;
        }

        chunks
    }

    /// Finds the best boundary near the ideal end position.
    fn find_best_boundary(
        &self,
        text: &str,
        start: usize,
        ideal_end: usize,
        boundaries: &[usize],
    ) -> usize {
        // If we're at the end of text, use that
        if ideal_end >= text.len() {
            return text.len();
        }

        // Look for a code boundary near the ideal end
        let search_start = start + (ideal_end - start) / 2; // Start from halfway
        let search_end = (ideal_end + self.chunk_size / 4).min(text.len());

        // Find boundaries in the search range
        let candidates: Vec<usize> = boundaries
            .iter()
            .copied()
            .filter(|&b| b > search_start && b <= search_end)
            .collect();

        // Prefer a boundary closer to ideal_end
        #[allow(clippy::cast_possible_wrap)]
        if let Some(&boundary) = candidates
            .iter()
            .min_by_key(|&&b| (b as i64 - ideal_end as i64).abs())
        {
            return boundary;
        }

        // Fall back to line boundary
        if let Some(newline) = text[search_start..ideal_end].rfind('\n') {
            return search_start + newline + 1;
        }

        ideal_end
    }

    /// Finds the start position for overlap.
    #[allow(clippy::unused_self)]
    fn find_overlap_start(
        &self,
        text: &str,
        current_end: usize,
        overlap: usize,
        boundaries: &[usize],
    ) -> usize {
        let target = current_end.saturating_sub(overlap);

        // Try to find a boundary at or before the target
        if let Some(&boundary) = boundaries
            .iter()
            .rev()
            .find(|&&b| b <= target && b < current_end)
        {
            return boundary;
        }

        // Fall back to line boundary
        if let Some(newline) = text[..target.min(text.len())].rfind('\n') {
            return newline + 1;
        }

        target.min(current_end)
    }
}

impl Chunker for CodeChunker {
    fn chunk(
        &self,
        buffer_id: i64,
        text: &str,
        metadata: Option<&ChunkMetadata>,
    ) -> Result<Vec<Chunk>> {
        self.validate(metadata)?;

        if text.is_empty() {
            return Ok(vec![]);
        }

        let chunk_size = metadata.map_or(self.chunk_size, |m| {
            if m.chunk_size > 0 {
                m.chunk_size
            } else {
                self.chunk_size
            }
        });
        let overlap = metadata.map_or(self.overlap, |m| m.overlap);

        // Detect language
        let lang = Self::detect_language(metadata);

        // Find code structure boundaries
        let boundaries = self.find_boundaries(text, lang);

        // Chunk at boundaries
        Ok(self.chunk_at_boundaries(buffer_id, text, &boundaries, chunk_size, overlap))
    }

    fn name(&self) -> &'static str {
        "code"
    }

    fn description(&self) -> &'static str {
        "Code-aware chunking at function/class boundaries"
    }
}

/// Supported programming languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    C,
    Cpp,
    Ruby,
    Php,
    Unknown,
}

impl Language {
    /// Detects language from file extension.
    fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Self::Rust,
            "py" | "pyw" | "pyi" => Self::Python,
            "js" | "mjs" | "cjs" | "jsx" => Self::JavaScript,
            "ts" | "tsx" | "mts" | "cts" => Self::TypeScript,
            "go" => Self::Go,
            "java" => Self::Java,
            "c" | "h" => Self::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => Self::Cpp,
            "rb" | "rake" | "gemspec" => Self::Ruby,
            "php" | "phtml" => Self::Php,
            _ => Self::Unknown,
        }
    }

    /// Returns regex patterns for detecting code boundaries.
    fn boundary_patterns(self) -> Vec<BoundaryPattern> {
        match self {
            Self::Rust => vec![
                BoundaryPattern::RustFn,
                BoundaryPattern::RustImpl,
                BoundaryPattern::RustStruct,
                BoundaryPattern::RustEnum,
                BoundaryPattern::RustTrait,
                BoundaryPattern::RustMod,
            ],
            Self::Python => vec![
                BoundaryPattern::PythonDef,
                BoundaryPattern::PythonClass,
                BoundaryPattern::PythonAsync,
            ],
            Self::JavaScript | Self::TypeScript => vec![
                BoundaryPattern::JsFunction,
                BoundaryPattern::JsClass,
                BoundaryPattern::JsArrowNamed,
                BoundaryPattern::JsMethod,
            ],
            Self::Go => vec![BoundaryPattern::GoFunc, BoundaryPattern::GoType],
            Self::Java => vec![
                BoundaryPattern::JavaClass,
                BoundaryPattern::JavaMethod,
                BoundaryPattern::JavaInterface,
            ],
            Self::C | Self::Cpp => vec![
                BoundaryPattern::CFunction,
                BoundaryPattern::CppClass,
                BoundaryPattern::CppNamespace,
            ],
            Self::Ruby => vec![
                BoundaryPattern::RubyDef,
                BoundaryPattern::RubyClass,
                BoundaryPattern::RubyModule,
            ],
            Self::Php => vec![BoundaryPattern::PhpFunction, BoundaryPattern::PhpClass],
            Self::Unknown => vec![BoundaryPattern::GenericFunction],
        }
    }
}

/// Patterns for detecting code structure boundaries.
#[derive(Debug, Clone, Copy)]
enum BoundaryPattern {
    // Rust patterns
    RustFn,
    RustImpl,
    RustStruct,
    RustEnum,
    RustTrait,
    RustMod,

    // Python patterns
    PythonDef,
    PythonClass,
    PythonAsync,

    // JavaScript/TypeScript patterns
    JsFunction,
    JsClass,
    JsArrowNamed,
    JsMethod,

    // Go patterns
    GoFunc,
    GoType,

    // Java patterns
    JavaClass,
    JavaMethod,
    JavaInterface,

    // C/C++ patterns
    CFunction,
    CppClass,
    CppNamespace,

    // Ruby patterns
    RubyDef,
    RubyClass,
    RubyModule,

    // PHP patterns
    PhpFunction,
    PhpClass,

    // Generic fallback
    GenericFunction,
}

impl BoundaryPattern {
    /// Returns the compiled regex for this pattern.
    fn regex(self) -> &'static Regex {
        macro_rules! static_regex {
            ($name:ident, $pattern:expr) => {{
                static $name: OnceLock<Regex> = OnceLock::new();
                $name.get_or_init(|| Regex::new($pattern).expect("valid regex"))
            }};
        }

        match self {
            // Rust
            Self::RustFn => static_regex!(
                RUST_FN,
                r"(?m)^[ \t]*(pub(\s*\([^)]*\))?\s+)?(async\s+)?(unsafe\s+)?(extern\s+\S+\s+)?fn\s+\w+"
            ),
            Self::RustImpl => static_regex!(RUST_IMPL, r"(?m)^[ \t]*(unsafe\s+)?impl(<[^>]*>)?\s+"),
            Self::RustStruct => static_regex!(
                RUST_STRUCT,
                r"(?m)^[ \t]*(pub(\s*\([^)]*\))?\s+)?struct\s+\w+"
            ),
            Self::RustEnum => {
                static_regex!(RUST_ENUM, r"(?m)^[ \t]*(pub(\s*\([^)]*\))?\s+)?enum\s+\w+")
            }
            Self::RustTrait => static_regex!(
                RUST_TRAIT,
                r"(?m)^[ \t]*(pub(\s*\([^)]*\))?\s+)?(unsafe\s+)?trait\s+\w+"
            ),
            Self::RustMod => {
                static_regex!(RUST_MOD, r"(?m)^[ \t]*(pub(\s*\([^)]*\))?\s+)?mod\s+\w+")
            }

            // Python
            Self::PythonDef => static_regex!(PYTHON_DEF, r"(?m)^[ \t]*def\s+\w+"),
            Self::PythonClass => static_regex!(PYTHON_CLASS, r"(?m)^[ \t]*class\s+\w+"),
            Self::PythonAsync => static_regex!(PYTHON_ASYNC, r"(?m)^[ \t]*async\s+def\s+\w+"),

            // JavaScript/TypeScript
            Self::JsFunction => static_regex!(
                JS_FUNCTION,
                r"(?m)^[ \t]*(export\s+)?(async\s+)?function\s*\*?\s*\w+"
            ),
            Self::JsClass => static_regex!(
                JS_CLASS,
                r"(?m)^[ \t]*(export\s+)?(abstract\s+)?class\s+\w+"
            ),
            Self::JsArrowNamed => static_regex!(
                JS_ARROW,
                r"(?m)^[ \t]*(export\s+)?(const|let|var)\s+\w+\s*=\s*(async\s+)?\([^)]*\)\s*=>"
            ),
            Self::JsMethod => static_regex!(
                JS_METHOD,
                r"(?m)^[ \t]*(static\s+)?(async\s+)?(get\s+|set\s+)?\w+\s*\([^)]*\)\s*\{"
            ),

            // Go
            Self::GoFunc => static_regex!(GO_FUNC, r"(?m)^func\s+(\([^)]+\)\s*)?\w+"),
            Self::GoType => static_regex!(GO_TYPE, r"(?m)^type\s+\w+\s+(struct|interface)"),

            // Java
            Self::JavaClass => static_regex!(
                JAVA_CLASS,
                r"(?m)^[ \t]*(public|private|protected)?\s*(abstract\s+)?(final\s+)?class\s+\w+"
            ),
            Self::JavaMethod => static_regex!(
                JAVA_METHOD,
                r"(?m)^[ \t]*(public|private|protected)\s+(static\s+)?(\w+\s+)+\w+\s*\([^)]*\)\s*(\{|throws)"
            ),
            Self::JavaInterface => {
                static_regex!(JAVA_INTERFACE, r"(?m)^[ \t]*(public\s+)?interface\s+\w+")
            }

            // C/C++
            Self::CFunction => static_regex!(
                C_FUNCTION,
                r"(?m)^[ \t]*(\w+\s+)+\**\s*\w+\s*\([^)]*\)\s*\{"
            ),
            Self::CppClass => static_regex!(
                CPP_CLASS,
                r"(?m)^[ \t]*(template\s*<[^>]*>\s*)?(class|struct)\s+\w+"
            ),
            Self::CppNamespace => static_regex!(CPP_NAMESPACE, r"(?m)^[ \t]*namespace\s+\w+"),

            // Ruby
            Self::RubyDef => static_regex!(RUBY_DEF, r"(?m)^[ \t]*def\s+\w+"),
            Self::RubyClass => static_regex!(RUBY_CLASS, r"(?m)^[ \t]*class\s+\w+"),
            Self::RubyModule => static_regex!(RUBY_MODULE, r"(?m)^[ \t]*module\s+\w+"),

            // PHP
            Self::PhpFunction => static_regex!(
                PHP_FUNCTION,
                r"(?m)^[ \t]*(public|private|protected)?\s*(static\s+)?function\s+\w+"
            ),
            Self::PhpClass => {
                static_regex!(PHP_CLASS, r"(?m)^[ \t]*(abstract\s+|final\s+)?class\s+\w+")
            }

            // Generic
            Self::GenericFunction => static_regex!(
                GENERIC_FUNCTION,
                r"(?m)^[ \t]*(function|def|fn|func|sub|proc)\s+\w+"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_chunker_new() {
        let chunker = CodeChunker::new();
        assert_eq!(chunker.name(), "code");
        assert_eq!(chunker.chunk_size, DEFAULT_CHUNK_SIZE);
    }

    #[test]
    fn test_code_chunker_with_size() {
        let chunker = CodeChunker::with_size(1000);
        assert_eq!(chunker.chunk_size, 1000);
        assert_eq!(chunker.overlap, DEFAULT_OVERLAP);
    }

    #[test]
    fn test_detect_language_rust() {
        let meta = ChunkMetadata::new().content_type("rs");
        let lang = CodeChunker::detect_language(Some(&meta));
        assert_eq!(lang, Language::Rust);
    }

    #[test]
    fn test_detect_language_from_source() {
        let meta = ChunkMetadata::new().source("src/main.py");
        let lang = CodeChunker::detect_language(Some(&meta));
        assert_eq!(lang, Language::Python);
    }

    #[test]
    fn test_detect_language_unknown() {
        let meta = ChunkMetadata::new().content_type("xyz");
        let lang = CodeChunker::detect_language(Some(&meta));
        assert_eq!(lang, Language::Unknown);
    }

    #[test]
    fn test_chunk_rust_code() {
        let chunker = CodeChunker::with_size(200);
        let code = r#"
fn main() {
    println!("Hello");
}

fn helper() {
    println!("Helper");
}

pub fn public_fn() {
    println!("Public");
}
"#;

        let meta = ChunkMetadata::with_size(200).content_type("rs");
        let chunks = chunker.chunk(1, code, Some(&meta)).unwrap();

        assert!(!chunks.is_empty());
        // Each function should ideally be in its own chunk
        for chunk in &chunks {
            assert!(!chunk.content.trim().is_empty());
        }
    }

    #[test]
    fn test_chunk_python_code() {
        let chunker = CodeChunker::with_size(150);
        let code = r#"
def main():
    print("Hello")

class MyClass:
    def method(self):
        pass

async def async_func():
    await something()
"#;

        let meta = ChunkMetadata::with_size(150).content_type("py");
        let chunks = chunker.chunk(1, code, Some(&meta)).unwrap();

        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_chunk_javascript_code() {
        let chunker = CodeChunker::with_size(200);
        let code = r#"
function greet(name) {
    console.log("Hello " + name);
}

class Person {
    constructor(name) {
        this.name = name;
    }
}

const arrow = (x) => x * 2;

export async function fetchData() {
    return await fetch("/api");
}
"#;

        let meta = ChunkMetadata::with_size(200).content_type("js");
        let chunks = chunker.chunk(1, code, Some(&meta)).unwrap();

        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_chunk_empty_text() {
        let chunker = CodeChunker::new();
        let chunks = chunker.chunk(1, "", None).unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_chunk_unknown_language() {
        let chunker = CodeChunker::with_size(100);
        let code = "some random text without code structure";

        let chunks = chunker.chunk(1, code, None).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_boundary_patterns_rust() {
        let patterns = Language::Rust.boundary_patterns();
        assert!(!patterns.is_empty());

        let code = "pub fn my_function() {}";
        let re = BoundaryPattern::RustFn.regex();
        assert!(re.is_match(code));
    }

    #[test]
    fn test_boundary_patterns_python() {
        let code = "def my_function():";
        let re = BoundaryPattern::PythonDef.regex();
        assert!(re.is_match(code));

        let code = "class MyClass:";
        let re = BoundaryPattern::PythonClass.regex();
        assert!(re.is_match(code));
    }

    #[test]
    fn test_language_extensions() {
        assert_eq!(Language::from_extension("rs"), Language::Rust);
        assert_eq!(Language::from_extension("py"), Language::Python);
        assert_eq!(Language::from_extension("js"), Language::JavaScript);
        assert_eq!(Language::from_extension("ts"), Language::TypeScript);
        assert_eq!(Language::from_extension("go"), Language::Go);
        assert_eq!(Language::from_extension("java"), Language::Java);
        assert_eq!(Language::from_extension("c"), Language::C);
        assert_eq!(Language::from_extension("cpp"), Language::Cpp);
        assert_eq!(Language::from_extension("rb"), Language::Ruby);
        assert_eq!(Language::from_extension("php"), Language::Php);
        assert_eq!(Language::from_extension("unknown"), Language::Unknown);
    }

    #[test]
    fn test_chunker_description() {
        let chunker = CodeChunker::new();
        assert!(!chunker.description().is_empty());
    }
}
