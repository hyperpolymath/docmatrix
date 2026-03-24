// SPDX-License-Identifier: PMPL-1.0-or-later
//! Parser and Renderer traits for format handlers

use crate::ast::{Document, SourceFormat};
use std::collections::HashMap;
use std::io::{Read, Write};

/// Error type for parsing and rendering
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Parse error at line {line}, column {column}: {message}")]
    ParseError {
        line: u32,
        column: u32,
        message: String,
    },

    #[error("Unsupported feature: {feature} in format {format:?}")]
    UnsupportedFeature {
        format: SourceFormat,
        feature: String,
    },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, ConversionError>;

/// Configuration for parsing
#[derive(Debug, Clone, Default)]
pub struct ParseConfig {
    /// Preserve source spans for error reporting
    pub preserve_spans: bool,
    /// Keep raw source for lossless round-trip
    pub preserve_raw_source: bool,
    /// Custom front matter delimiter (default: "---")
    pub front_matter_delimiter: Option<String>,
    /// Format-specific options
    pub format_options: HashMap<String, String>,
}

/// Configuration for rendering
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Target line width for wrapping (0 = no wrap)
    pub line_width: usize,
    /// Indentation string (default: 2 spaces)
    pub indent: String,
    /// Use hard line breaks
    pub hard_breaks: bool,
    /// Format-specific options
    pub format_options: HashMap<String, String>,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            line_width: 80,
            indent: "  ".to_string(),
            hard_breaks: false,
            format_options: HashMap::new(),
        }
    }
}

/// Parser trait: convert source format to AST
pub trait Parser: Send + Sync {
    /// The source format this parser handles
    fn format(&self) -> SourceFormat;

    /// Parse a string into a Document
    fn parse(&self, input: &str, config: &ParseConfig) -> Result<Document>;
}

/// Renderer trait: convert AST to target format
pub trait Renderer: Send + Sync {
    /// The target format this renderer produces
    fn format(&self) -> SourceFormat;

    /// Render a Document to a string
    fn render(&self, doc: &Document, config: &RenderConfig) -> Result<String>;
}

/// Extension trait for streaming operations (not dyn-compatible)
pub trait ParserExt: Parser {
    /// Parse from a reader (streaming when possible)
    fn parse_reader<R: Read>(&self, reader: R, config: &ParseConfig) -> Result<Document> {
        let mut input = String::new();
        let mut reader = reader;
        reader.read_to_string(&mut input)?;
        self.parse(&input, config)
    }
}

/// Extension trait for streaming operations (not dyn-compatible)
pub trait RendererExt: Renderer {
    /// Render to a writer (streaming when possible)
    fn render_writer<W: Write>(
        &self,
        doc: &Document,
        writer: &mut W,
        config: &RenderConfig,
    ) -> Result<()> {
        let output = self.render(doc, config)?;
        writer.write_all(output.as_bytes())?;
        Ok(())
    }
}

// Blanket implementations
impl<T: Parser> ParserExt for T {}
impl<T: Renderer> RendererExt for T {}

/// Combined parser + renderer for a format
pub trait FormatHandler: Parser + Renderer {
    /// Check if this format supports a specific feature
    fn supports_feature(&self, feature: &str) -> bool;

    /// Get list of supported features
    fn supported_features(&self) -> &[&str];
}

/// Registry of format handlers
pub struct FormatRegistry {
    handlers: HashMap<SourceFormat, Box<dyn FormatHandler>>,
}

impl FormatRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, handler: Box<dyn FormatHandler>) {
        let format = Parser::format(handler.as_ref());
        self.handlers.insert(format, handler);
    }

    pub fn get(&self, format: SourceFormat) -> Option<&dyn FormatHandler> {
        self.handlers.get(&format).map(|h| h.as_ref())
    }

    /// Convert between formats
    pub fn convert(
        &self,
        input: &str,
        from: SourceFormat,
        to: SourceFormat,
        parse_config: &ParseConfig,
        render_config: &RenderConfig,
    ) -> Result<String> {
        if from == to {
            return Ok(input.to_string());
        }

        let from_handler = self
            .get(from)
            .ok_or_else(|| ConversionError::UnsupportedFeature {
                format: from,
                feature: "parsing".to_string(),
            })?;

        let to_handler = self
            .get(to)
            .ok_or_else(|| ConversionError::UnsupportedFeature {
                format: to,
                feature: "rendering".to_string(),
            })?;

        let doc = from_handler.parse(input, parse_config)?;
        to_handler.render(&doc, render_config)
    }
}

impl Default for FormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}
