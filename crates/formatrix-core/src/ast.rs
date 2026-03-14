// SPDX-License-Identifier: PMPL-1.0-or-later
//! Unified document AST for multi-format conversion
//!
//! This AST is designed to be format-neutral while preserving semantic meaning.
//! Format-specific features that cannot be cleanly mapped are preserved as Raw nodes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Source format identifier for provenance tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceFormat {
    PlainText,
    Markdown,
    AsciiDoc,
    Djot,
    OrgMode,
    ReStructuredText,
    Typst,
}

impl SourceFormat {
    /// File extension for this format
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::PlainText => "txt",
            Self::Markdown => "md",
            Self::AsciiDoc => "adoc",
            Self::Djot => "dj",
            Self::OrgMode => "org",
            Self::ReStructuredText => "rst",
            Self::Typst => "typ",
        }
    }

    /// Short display name
    pub const fn label(&self) -> &'static str {
        match self {
            Self::PlainText => "TXT",
            Self::Markdown => "MD",
            Self::AsciiDoc => "ADOC",
            Self::Djot => "DJOT",
            Self::OrgMode => "ORG",
            Self::ReStructuredText => "RST",
            Self::Typst => "TYP",
        }
    }

    /// All formats in tab order
    pub const ALL: [Self; 7] = [
        Self::PlainText,
        Self::Markdown,
        Self::AsciiDoc,
        Self::Djot,
        Self::OrgMode,
        Self::ReStructuredText,
        Self::Typst,
    ];
}

/// Span information for source mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub column: u32,
}

/// Document metadata (front matter, properties)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentMeta {
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub date: Option<String>,
    pub language: Option<String>,
    /// Format-specific metadata preserved as key-value pairs
    pub custom: HashMap<String, MetaValue>,
}

/// Metadata value (recursive for nested structures)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetaValue {
    String(String),
    Bool(bool),
    Integer(i64),
    Float(f64),
    List(Vec<MetaValue>),
    Map(HashMap<String, MetaValue>),
}

/// The root document node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub source_format: SourceFormat,
    pub meta: DocumentMeta,
    pub content: Vec<Block>,
    /// Preserved raw source for lossless round-trip (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_source: Option<String>,
}

impl Document {
    /// Create a new empty document
    pub fn new(format: SourceFormat) -> Self {
        Self {
            source_format: format,
            meta: DocumentMeta::default(),
            content: Vec::new(),
            raw_source: None,
        }
    }

    /// Count words in the document
    pub fn word_count(&self) -> usize {
        self.content.iter().map(|b| b.word_count()).sum()
    }

    /// Count characters in the document
    pub fn char_count(&self) -> usize {
        self.content.iter().map(|b| b.char_count()).sum()
    }
}

/// Block-level elements (structural)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    /// Plain paragraph
    Paragraph {
        content: Vec<Inline>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },

    /// Heading with level 1-6
    Heading {
        level: u8,
        content: Vec<Inline>,
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },

    /// Code block with optional language
    CodeBlock {
        language: Option<String>,
        content: String,
        line_numbers: bool,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        highlight_lines: Vec<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },

    /// Block quote (may be nested)
    BlockQuote {
        content: Vec<Block>,
        attribution: Option<Vec<Inline>>,
        admonition: Option<AdmonitionType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },

    /// Ordered or unordered list
    List {
        kind: ListKind,
        items: Vec<ListItem>,
        start: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },

    /// Definition list (term + definitions)
    DefinitionList {
        items: Vec<DefinitionItem>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },

    /// Table
    Table {
        caption: Option<Vec<Inline>>,
        columns: Vec<ColumnSpec>,
        header: Option<TableRow>,
        body: Vec<TableRow>,
        footer: Option<TableRow>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },

    /// Horizontal rule / thematic break
    ThematicBreak {
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },

    /// Math block (display mode)
    MathBlock {
        content: String,
        notation: MathNotation,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },

    /// Generic container with attributes (div-like)
    Container {
        id: Option<String>,
        classes: Vec<String>,
        attributes: HashMap<String, String>,
        content: Vec<Block>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },

    /// Figure with caption
    Figure {
        content: Vec<Block>,
        caption: Option<Vec<Inline>>,
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },

    /// Raw content preserved from source format
    Raw {
        format: SourceFormat,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },

    /// Footnote definition
    FootnoteDefinition {
        label: String,
        content: Vec<Block>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },

    /// Table of contents placeholder
    TableOfContents {
        max_depth: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
}

impl Block {
    /// Count words in this block
    pub fn word_count(&self) -> usize {
        match self {
            Block::Paragraph { content, .. } => content.iter().map(|i| i.word_count()).sum(),
            Block::Heading { content, .. } => content.iter().map(|i| i.word_count()).sum(),
            Block::CodeBlock { content, .. } => {
                content.split_whitespace().count()
            }
            Block::BlockQuote { content, .. } => content.iter().map(|b| b.word_count()).sum(),
            Block::List { items, .. } => {
                items.iter().flat_map(|i| &i.content).map(|b| b.word_count()).sum()
            }
            Block::Container { content, .. } => content.iter().map(|b| b.word_count()).sum(),
            Block::Figure { content, caption, .. } => {
                let content_count: usize = content.iter().map(|b| b.word_count()).sum();
                let caption_count: usize = caption.as_ref().map_or(0, |c| c.iter().map(|i| i.word_count()).sum());
                content_count + caption_count
            }
            _ => 0,
        }
    }

    /// Count characters in this block
    pub fn char_count(&self) -> usize {
        match self {
            Block::Paragraph { content, .. } => content.iter().map(|i| i.char_count()).sum(),
            Block::Heading { content, .. } => content.iter().map(|i| i.char_count()).sum(),
            Block::CodeBlock { content, .. } => content.chars().count(),
            Block::BlockQuote { content, .. } => content.iter().map(|b| b.char_count()).sum(),
            Block::List { items, .. } => {
                items.iter().flat_map(|i| &i.content).map(|b| b.char_count()).sum()
            }
            Block::Container { content, .. } => content.iter().map(|b| b.char_count()).sum(),
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListKind {
    Bullet,
    Ordered,
    Task,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    pub content: Vec<Block>,
    pub checked: Option<bool>,
    pub marker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionItem {
    pub term: Vec<Inline>,
    pub definitions: Vec<Vec<Block>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColumnAlignment {
    Left,
    Center,
    Right,
    Default,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSpec {
    pub alignment: ColumnAlignment,
    pub width: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub content: Vec<Block>,
    pub colspan: u32,
    pub rowspan: u32,
    pub alignment: Option<ColumnAlignment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdmonitionType {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
    Danger,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MathNotation {
    LaTeX,
    AsciiMath,
    MathML,
}

/// Inline elements (character-level)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Inline {
    /// Plain text
    Text { content: String },

    /// Emphasis (usually italic)
    Emphasis { content: Vec<Inline> },

    /// Strong emphasis (usually bold)
    Strong { content: Vec<Inline> },

    /// Strikethrough
    Strikethrough { content: Vec<Inline> },

    /// Underline
    Underline { content: Vec<Inline> },

    /// Superscript
    Superscript { content: Vec<Inline> },

    /// Subscript
    Subscript { content: Vec<Inline> },

    /// Small caps
    SmallCaps { content: Vec<Inline> },

    /// Inline code
    Code { content: String, language: Option<String> },

    /// Inline math
    Math { content: String, notation: MathNotation },

    /// Hyperlink
    Link {
        url: String,
        title: Option<String>,
        content: Vec<Inline>,
        link_type: LinkType,
    },

    /// Image
    Image {
        url: String,
        alt: String,
        title: Option<String>,
        width: Option<String>,
        height: Option<String>,
    },

    /// Footnote reference
    FootnoteRef { label: String },

    /// Citation
    Citation {
        keys: Vec<String>,
        prefix: Option<Vec<Inline>>,
        suffix: Option<Vec<Inline>>,
    },

    /// Line break (hard break)
    LineBreak,

    /// Soft break (may become space or newline)
    SoftBreak,

    /// Non-breaking space
    NonBreakingSpace,

    /// Generic span with attributes
    Span {
        id: Option<String>,
        classes: Vec<String>,
        attributes: HashMap<String, String>,
        content: Vec<Inline>,
    },

    /// Raw inline content from source format
    RawInline { format: SourceFormat, content: String },

    /// Quoted text
    Quoted { quote_type: QuoteType, content: Vec<Inline> },

    /// Keyboard input
    Keyboard { content: String },

    /// Highlight/mark
    Highlight { content: Vec<Inline> },
}

impl Inline {
    /// Count words in this inline element
    pub fn word_count(&self) -> usize {
        match self {
            Inline::Text { content } => content.split_whitespace().count(),
            Inline::Emphasis { content }
            | Inline::Strong { content }
            | Inline::Strikethrough { content }
            | Inline::Underline { content }
            | Inline::Highlight { content } => content.iter().map(|i| i.word_count()).sum(),
            Inline::Code { content, .. } => content.split_whitespace().count(),
            _ => 0,
        }
    }

    /// Count characters in this inline element
    pub fn char_count(&self) -> usize {
        match self {
            Inline::Text { content } => content.chars().count(),
            Inline::Emphasis { content }
            | Inline::Strong { content }
            | Inline::Strikethrough { content }
            | Inline::Underline { content }
            | Inline::Highlight { content } => content.iter().map(|i| i.char_count()).sum(),
            Inline::Code { content, .. } => content.chars().count(),
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    Inline,
    Reference,
    AutoLink,
    WikiLink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuoteType {
    Single,
    Double,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_new() {
        let doc = Document::new(SourceFormat::Markdown);
        assert_eq!(doc.source_format, SourceFormat::Markdown);
        assert!(doc.content.is_empty());
    }

    #[test]
    fn test_word_count() {
        let doc = Document {
            source_format: SourceFormat::Markdown,
            meta: DocumentMeta::default(),
            content: vec![Block::Paragraph {
                content: vec![Inline::Text {
                    content: "Hello world this is a test".to_string(),
                }],
                span: None,
            }],
            raw_source: None,
        };
        assert_eq!(doc.word_count(), 6);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // Strategy for generating SourceFormat
    fn source_format_strategy() -> impl Strategy<Value = SourceFormat> {
        prop_oneof![
            Just(SourceFormat::PlainText),
            Just(SourceFormat::Markdown),
            Just(SourceFormat::AsciiDoc),
            Just(SourceFormat::Djot),
            Just(SourceFormat::OrgMode),
            Just(SourceFormat::ReStructuredText),
            Just(SourceFormat::Typst),
        ]
    }

    // Strategy for generating simple text (no special characters for JSON safety)
    fn simple_text_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 ]{0,100}".prop_map(|s| s.trim().to_string())
    }

    // Strategy for generating Inline::Text
    fn inline_text_strategy() -> impl Strategy<Value = Inline> {
        simple_text_strategy().prop_map(|content| Inline::Text { content })
    }

    // Strategy for generating simple Inlines (no recursion)
    fn simple_inline_strategy() -> impl Strategy<Value = Inline> {
        prop_oneof![
            inline_text_strategy(),
            Just(Inline::LineBreak),
            Just(Inline::SoftBreak),
            Just(Inline::NonBreakingSpace),
            simple_text_strategy().prop_map(|content| Inline::Code {
                content,
                language: None
            }),
        ]
    }

    // Strategy for ListKind
    fn list_kind_strategy() -> impl Strategy<Value = ListKind> {
        prop_oneof![
            Just(ListKind::Bullet),
            Just(ListKind::Ordered),
            Just(ListKind::Task),
        ]
    }

    // Strategy for generating Paragraphs
    fn paragraph_strategy() -> impl Strategy<Value = Block> {
        prop::collection::vec(simple_inline_strategy(), 0..5).prop_map(|content| {
            Block::Paragraph { content, span: None }
        })
    }

    // Strategy for generating Headings
    fn heading_strategy() -> impl Strategy<Value = Block> {
        (1u8..=6, prop::collection::vec(simple_inline_strategy(), 1..4)).prop_map(
            |(level, content)| Block::Heading {
                level,
                content,
                id: None,
                span: None,
            },
        )
    }

    // Strategy for generating CodeBlocks
    fn code_block_strategy() -> impl Strategy<Value = Block> {
        (
            proptest::option::of("[a-z]+"),
            simple_text_strategy(),
            proptest::bool::ANY,
        )
            .prop_map(|(language, content, line_numbers)| Block::CodeBlock {
                language,
                content,
                line_numbers,
                highlight_lines: Vec::new(),
                span: None,
            })
    }

    // Strategy for generating simple blocks (no deep recursion)
    fn simple_block_strategy() -> impl Strategy<Value = Block> {
        prop_oneof![
            paragraph_strategy(),
            heading_strategy(),
            code_block_strategy(),
            Just(Block::ThematicBreak { span: None }),
        ]
    }

    // Strategy for generating Documents
    fn document_strategy() -> impl Strategy<Value = Document> {
        (
            source_format_strategy(),
            prop::collection::vec(simple_block_strategy(), 0..10),
        )
            .prop_map(|(source_format, content)| Document {
                source_format,
                meta: DocumentMeta::default(),
                content,
                raw_source: None,
            })
    }

    // Strategy for MetaValue
    fn meta_value_strategy() -> impl Strategy<Value = MetaValue> {
        prop_oneof![
            simple_text_strategy().prop_map(MetaValue::String),
            proptest::bool::ANY.prop_map(MetaValue::Bool),
            (-1000i64..1000).prop_map(MetaValue::Integer),
            (-1000.0f64..1000.0).prop_map(MetaValue::Float),
        ]
    }

    proptest! {
        // Property: All SourceFormats have non-empty extensions
        #[test]
        fn prop_source_format_has_extension(format in source_format_strategy()) {
            prop_assert!(!format.extension().is_empty());
        }

        // Property: All SourceFormats have non-empty labels
        #[test]
        fn prop_source_format_has_label(format in source_format_strategy()) {
            prop_assert!(!format.label().is_empty());
        }

        // Property: Extension and label are different (except edge cases)
        #[test]
        fn prop_source_format_extension_differs_from_label(format in source_format_strategy()) {
            // Extensions are lowercase file extensions, labels are display names
            prop_assert!(format.extension() != format.label() || format.extension() == format.label().to_lowercase());
        }

        // Property: Document word_count is non-negative
        #[test]
        fn prop_document_word_count_nonnegative(doc in document_strategy()) {
            prop_assert!(doc.word_count() >= 0);
        }

        // Property: Document char_count is non-negative
        #[test]
        fn prop_document_char_count_nonnegative(doc in document_strategy()) {
            prop_assert!(doc.char_count() >= 0);
        }

        // Property: Empty document has zero word count
        #[test]
        fn prop_empty_document_zero_words(format in source_format_strategy()) {
            let doc = Document::new(format);
            prop_assert_eq!(doc.word_count(), 0);
        }

        // Property: Empty document has zero char count
        #[test]
        fn prop_empty_document_zero_chars(format in source_format_strategy()) {
            let doc = Document::new(format);
            prop_assert_eq!(doc.char_count(), 0);
        }

        // Property: Document serialization roundtrip
        #[test]
        fn prop_document_serde_roundtrip(doc in document_strategy()) {
            let json = serde_json::to_string(&doc).expect("serialize");
            let deserialized: Document = serde_json::from_str(&json).expect("deserialize");
            prop_assert_eq!(doc.source_format, deserialized.source_format);
            prop_assert_eq!(doc.content.len(), deserialized.content.len());
        }

        // Property: Inline Text word count equals split_whitespace count
        #[test]
        fn prop_inline_text_word_count(content in simple_text_strategy()) {
            let inline = Inline::Text { content: content.clone() };
            prop_assert_eq!(inline.word_count(), content.split_whitespace().count());
        }

        // Property: Inline Text char count equals chars().count()
        #[test]
        fn prop_inline_text_char_count(content in simple_text_strategy()) {
            let inline = Inline::Text { content: content.clone() };
            prop_assert_eq!(inline.char_count(), content.chars().count());
        }

        // Property: Heading level is always 1-6
        #[test]
        fn prop_heading_level_in_range(level in 1u8..=6, content in prop::collection::vec(simple_inline_strategy(), 1..4)) {
            let block = Block::Heading {
                level,
                content,
                id: None,
                span: None,
            };
            if let Block::Heading { level: l, .. } = block {
                prop_assert!(l >= 1 && l <= 6);
            }
        }

        // Property: MetaValue serialization roundtrip
        #[test]
        fn prop_meta_value_serde_roundtrip(value in meta_value_strategy()) {
            let json = serde_json::to_string(&value).expect("serialize");
            let deserialized: MetaValue = serde_json::from_str(&json).expect("deserialize");
            match (&value, &deserialized) {
                (MetaValue::String(a), MetaValue::String(b)) => prop_assert_eq!(a, b),
                (MetaValue::Bool(a), MetaValue::Bool(b)) => prop_assert_eq!(a, b),
                (MetaValue::Integer(a), MetaValue::Integer(b)) => prop_assert_eq!(a, b),
                (MetaValue::Float(a), MetaValue::Float(b)) => {
                    // Float comparison with tolerance
                    prop_assert!((a - b).abs() < 0.0001);
                }
                _ => prop_assert!(false, "Type mismatch after roundtrip"),
            }
        }

        // Property: Block paragraph word count is sum of inline word counts
        #[test]
        fn prop_paragraph_word_count_sum(inlines in prop::collection::vec(inline_text_strategy(), 0..5)) {
            let expected: usize = inlines.iter().map(|i| i.word_count()).sum();
            let block = Block::Paragraph { content: inlines, span: None };
            prop_assert_eq!(block.word_count(), expected);
        }

        // Property: ListKind serialization roundtrip
        #[test]
        fn prop_list_kind_serde_roundtrip(kind in list_kind_strategy()) {
            let json = serde_json::to_string(&kind).expect("serialize");
            let deserialized: ListKind = serde_json::from_str(&json).expect("deserialize");
            prop_assert_eq!(kind, deserialized);
        }

        // Property: SourceFormat::ALL contains all variants
        #[test]
        fn prop_source_format_all_contains(format in source_format_strategy()) {
            prop_assert!(SourceFormat::ALL.contains(&format));
        }
    }
}
