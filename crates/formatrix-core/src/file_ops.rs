// SPDX-License-Identifier: AGPL-3.0-or-later
//! File operations for document loading and saving (FD-M06)
//!
//! Provides:
//! - File opening with automatic format detection
//! - File saving with format selection
//! - Path-based format detection from extensions
//! - Content-based format detection heuristics

use crate::ast::{Document, SourceFormat};
use crate::formats::{
    AsciidocHandler, DjotHandler, MarkdownHandler, OrgModeHandler, PlainTextHandler, RstHandler,
    TypstHandler,
};
use crate::traits::{ParseConfig, Parser, RenderConfig, Renderer};
use std::fs;
use std::path::Path;
use thiserror::Error;

/// File operation errors
#[derive(Debug, Error)]
pub enum FileError {
    /// IO error during file operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Format detection failed
    #[error("Could not detect format for file: {path}")]
    UnknownFormat { path: String },

    /// Unsupported format for operation
    #[error("Unsupported format: {format:?}")]
    UnsupportedFormat { format: SourceFormat },

    /// Parse error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Render error
    #[error("Render error: {0}")]
    Render(String),
}

impl From<crate::traits::ConversionError> for FileError {
    fn from(err: crate::traits::ConversionError) -> Self {
        match err {
            crate::traits::ConversionError::ParseError { message, .. } => FileError::Parse(message),
            crate::traits::ConversionError::IoError(e) => FileError::Io(e),
            crate::traits::ConversionError::UnsupportedFeature { format, .. } => {
                FileError::UnsupportedFormat { format }
            }
            crate::traits::ConversionError::SerializationError(msg) => FileError::Render(msg),
        }
    }
}

/// Result type for file operations
pub type FileResult<T> = std::result::Result<T, FileError>;

/// Metadata about an opened file
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Full path to the file
    pub path: String,
    /// Detected or specified format
    pub format: SourceFormat,
    /// File size in bytes
    pub size: u64,
    /// Whether the file is read-only
    pub read_only: bool,
}

/// Opened document with file metadata
#[derive(Debug, Clone)]
pub struct OpenedDocument {
    /// The parsed document
    pub document: Document,
    /// File information
    pub file_info: FileInfo,
}

/// Detect format from file extension
pub fn format_from_extension(path: &Path) -> Option<SourceFormat> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "txt" | "text" => Some(SourceFormat::PlainText),
        "md" | "markdown" | "mdown" | "mkd" => Some(SourceFormat::Markdown),
        "adoc" | "asciidoc" | "asc" => Some(SourceFormat::AsciiDoc),
        "dj" | "djot" => Some(SourceFormat::Djot),
        "org" => Some(SourceFormat::OrgMode),
        "rst" | "rest" | "restructuredtext" => Some(SourceFormat::ReStructuredText),
        "typ" | "typst" => Some(SourceFormat::Typst),
        _ => None,
    }
}

/// Detect format from content using heuristics
pub fn format_from_content(content: &str) -> SourceFormat {
    let trimmed = content.trim();

    // Check for org-mode markers (most specific first)
    if trimmed.starts_with("#+") || trimmed.contains("\n#+") {
        return SourceFormat::OrgMode;
    }

    // Check for AsciiDoc markers
    if trimmed.starts_with("= ") && !trimmed.starts_with("= {") {
        return SourceFormat::AsciiDoc;
    }
    if trimmed.starts_with(":toc:") || trimmed.contains("\n:toc:") {
        return SourceFormat::AsciiDoc;
    }

    // Check for Typst markers
    if trimmed.contains("#let ") || trimmed.contains("#set ") || trimmed.contains("#show ") {
        return SourceFormat::Typst;
    }
    if trimmed.starts_with("#[") || trimmed.contains("\n#[") {
        return SourceFormat::Typst;
    }

    // Check for RST markers
    if trimmed.contains(".. ") && (trimmed.contains("::") || trimmed.contains(".. code-block::")) {
        return SourceFormat::ReStructuredText;
    }
    // RST title underlines
    if trimmed.lines().any(|line| {
        let chars: Vec<char> = line.chars().collect();
        chars.len() > 3
            && chars.iter().all(|&c| c == '=' || c == '-' || c == '~' || c == '^')
    }) {
        return SourceFormat::ReStructuredText;
    }

    // Check for Djot markers
    if trimmed.contains("{.") || trimmed.contains("[^") {
        return SourceFormat::Djot;
    }

    // Check for Markdown markers (most common, check last)
    if trimmed.starts_with("# ") || trimmed.contains("\n# ") {
        return SourceFormat::Markdown;
    }
    if trimmed.contains("```") || trimmed.contains("~~~") {
        return SourceFormat::Markdown;
    }
    if trimmed.contains("[](") || trimmed.contains("![](") {
        return SourceFormat::Markdown;
    }

    // Default to plain text
    SourceFormat::PlainText
}

/// Open a file and parse it to a Document
///
/// Format is detected from file extension first, then from content if needed.
pub fn open_file(path: impl AsRef<Path>) -> FileResult<OpenedDocument> {
    open_file_with_config(path, &ParseConfig::default())
}

/// Open a file with custom parse configuration
pub fn open_file_with_config(
    path: impl AsRef<Path>,
    config: &ParseConfig,
) -> FileResult<OpenedDocument> {
    let path = path.as_ref();

    // Read file content
    let content = fs::read_to_string(path)?;

    // Get file metadata
    let metadata = fs::metadata(path)?;
    let size = metadata.len();
    let read_only = metadata.permissions().readonly();

    // Detect format
    let format = format_from_extension(path).unwrap_or_else(|| format_from_content(&content));

    // Parse based on format
    let document = parse_content(&content, format, config)?;

    Ok(OpenedDocument {
        document,
        file_info: FileInfo {
            path: path.to_string_lossy().to_string(),
            format,
            size,
            read_only,
        },
    })
}

/// Open a file with explicit format specification
pub fn open_file_as(
    path: impl AsRef<Path>,
    format: SourceFormat,
    config: &ParseConfig,
) -> FileResult<OpenedDocument> {
    let path = path.as_ref();

    // Read file content
    let content = fs::read_to_string(path)?;

    // Get file metadata
    let metadata = fs::metadata(path)?;
    let size = metadata.len();
    let read_only = metadata.permissions().readonly();

    // Parse with specified format
    let document = parse_content(&content, format, config)?;

    Ok(OpenedDocument {
        document,
        file_info: FileInfo {
            path: path.to_string_lossy().to_string(),
            format,
            size,
            read_only,
        },
    })
}

/// Parse content string to Document
fn parse_content(content: &str, format: SourceFormat, config: &ParseConfig) -> FileResult<Document> {
    let doc = match format {
        SourceFormat::PlainText => PlainTextHandler::new().parse(content, config)?,
        SourceFormat::Markdown => MarkdownHandler::new().parse(content, config)?,
        SourceFormat::AsciiDoc => AsciidocHandler::new().parse(content, config)?,
        SourceFormat::Djot => DjotHandler::new().parse(content, config)?,
        SourceFormat::OrgMode => OrgModeHandler::new().parse(content, config)?,
        SourceFormat::ReStructuredText => RstHandler::new().parse(content, config)?,
        SourceFormat::Typst => TypstHandler::new().parse(content, config)?,
    };
    Ok(doc)
}

/// Save a document to a file
///
/// Format is determined from the file extension.
pub fn save_file(doc: &Document, path: impl AsRef<Path>) -> FileResult<()> {
    save_file_with_config(doc, path, &RenderConfig::default())
}

/// Save a document with custom render configuration
pub fn save_file_with_config(
    doc: &Document,
    path: impl AsRef<Path>,
    config: &RenderConfig,
) -> FileResult<()> {
    let path = path.as_ref();

    // Detect format from extension, or use document's source format
    let format = format_from_extension(path).unwrap_or(doc.source_format);

    // Render and save
    save_file_as(doc, path, format, config)
}

/// Save a document to a file with explicit format
pub fn save_file_as(
    doc: &Document,
    path: impl AsRef<Path>,
    format: SourceFormat,
    config: &RenderConfig,
) -> FileResult<()> {
    let path = path.as_ref();

    // Render content
    let content = render_content(doc, format, config)?;

    // Write to file
    fs::write(path, content)?;

    Ok(())
}

/// Render document to string
fn render_content(doc: &Document, format: SourceFormat, config: &RenderConfig) -> FileResult<String> {
    let output = match format {
        SourceFormat::PlainText => PlainTextHandler::new().render(doc, config)?,
        SourceFormat::Markdown => MarkdownHandler::new().render(doc, config)?,
        SourceFormat::AsciiDoc => AsciidocHandler::new().render(doc, config)?,
        SourceFormat::Djot => DjotHandler::new().render(doc, config)?,
        SourceFormat::OrgMode => OrgModeHandler::new().render(doc, config)?,
        SourceFormat::ReStructuredText => RstHandler::new().render(doc, config)?,
        SourceFormat::Typst => TypstHandler::new().render(doc, config)?,
    };
    Ok(output)
}

/// Convert a file from one format to another
pub fn convert_file(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> FileResult<()> {
    convert_file_with_config(
        input_path,
        output_path,
        &ParseConfig::default(),
        &RenderConfig::default(),
    )
}

/// Convert a file with custom configuration
pub fn convert_file_with_config(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    parse_config: &ParseConfig,
    render_config: &RenderConfig,
) -> FileResult<()> {
    // Open and parse input
    let opened = open_file_with_config(input_path, parse_config)?;

    // Save to output (format detected from extension)
    save_file_with_config(&opened.document, output_path, render_config)?;

    Ok(())
}

/// Get the default file extension for a format
pub fn extension_for_format(format: SourceFormat) -> &'static str {
    format.extension()
}

/// Get all supported file extensions
pub fn supported_extensions() -> &'static [&'static str] {
    &[
        "txt", "text", "md", "markdown", "mdown", "mkd", "adoc", "asciidoc", "asc", "dj", "djot",
        "org", "rst", "rest", "restructuredtext", "typ", "typst",
    ]
}

/// Check if a file extension is supported
pub fn is_supported_extension(ext: &str) -> bool {
    let ext_lower = ext.to_lowercase();
    supported_extensions().contains(&ext_lower.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_format_from_extension() {
        assert_eq!(
            format_from_extension(Path::new("test.md")),
            Some(SourceFormat::Markdown)
        );
        assert_eq!(
            format_from_extension(Path::new("test.org")),
            Some(SourceFormat::OrgMode)
        );
        assert_eq!(
            format_from_extension(Path::new("test.adoc")),
            Some(SourceFormat::AsciiDoc)
        );
        assert_eq!(
            format_from_extension(Path::new("test.rst")),
            Some(SourceFormat::ReStructuredText)
        );
        assert_eq!(
            format_from_extension(Path::new("test.typ")),
            Some(SourceFormat::Typst)
        );
        assert_eq!(
            format_from_extension(Path::new("test.dj")),
            Some(SourceFormat::Djot)
        );
        assert_eq!(
            format_from_extension(Path::new("test.txt")),
            Some(SourceFormat::PlainText)
        );
        assert_eq!(format_from_extension(Path::new("test.xyz")), None);
    }

    #[test]
    fn test_format_from_content() {
        assert_eq!(
            format_from_content("# Heading\n\nParagraph"),
            SourceFormat::Markdown
        );
        assert_eq!(
            format_from_content("#+TITLE: Test\n* Heading"),
            SourceFormat::OrgMode
        );
        assert_eq!(
            format_from_content("= Document Title\n\nContent"),
            SourceFormat::AsciiDoc
        );
        assert_eq!(
            format_from_content("#let x = 1\nContent"),
            SourceFormat::Typst
        );
        assert_eq!(
            format_from_content("Just plain text"),
            SourceFormat::PlainText
        );
    }

    #[test]
    fn test_open_and_save_markdown() {
        // Create a temp file with markdown content
        let mut temp = NamedTempFile::with_suffix(".md").unwrap();
        writeln!(temp, "# Test Heading\n\nThis is a paragraph.").unwrap();

        // Open the file
        let opened = open_file(temp.path()).unwrap();
        assert_eq!(opened.file_info.format, SourceFormat::Markdown);
        assert!(opened.document.meta.title.is_some() || !opened.document.content.is_empty());

        // Save to a new file
        let output = NamedTempFile::with_suffix(".md").unwrap();
        save_file(&opened.document, output.path()).unwrap();

        // Verify the output exists and has content
        let saved_content = fs::read_to_string(output.path()).unwrap();
        assert!(!saved_content.is_empty());
    }

    #[test]
    fn test_convert_file() {
        // Create a temp markdown file
        let mut input = NamedTempFile::with_suffix(".md").unwrap();
        writeln!(input, "# Hello\n\nWorld").unwrap();

        // Convert to org-mode
        let output = NamedTempFile::with_suffix(".org").unwrap();
        convert_file(input.path(), output.path()).unwrap();

        // Verify output
        let content = fs::read_to_string(output.path()).unwrap();
        assert!(content.contains("Hello") || content.contains("World"));
    }

    #[test]
    fn test_supported_extensions() {
        assert!(is_supported_extension("md"));
        assert!(is_supported_extension("MD"));
        assert!(is_supported_extension("org"));
        assert!(is_supported_extension("adoc"));
        assert!(!is_supported_extension("docx"));
        assert!(!is_supported_extension("pdf"));
    }
}
