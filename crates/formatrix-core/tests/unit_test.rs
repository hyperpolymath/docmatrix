// SPDX-License-Identifier: PMPL-1.0-or-later
//! Comprehensive unit tests for formatrix-core

use formatrix_core::{
    ast::{Block, Document, DocumentMeta, Inline, SourceFormat, MetaValue},
    traits::{FormatHandler, Parser, ParseConfig, RenderConfig, Renderer, FormatRegistry},
    formats::PlainTextHandler,
};
use std::collections::HashMap;

// ============================================================================
// Parser Tests
// ============================================================================

#[test]
fn test_parser_simple_text() {
    let parser = PlainTextHandler::new();
    let input = "Hello world";
    let config = ParseConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    assert_eq!(doc.source_format, SourceFormat::PlainText);
    assert_eq!(doc.content.len(), 1);
}

#[test]
fn test_parser_multiple_paragraphs() {
    let parser = PlainTextHandler::new();
    let input = "Para 1\n\nPara 2\n\nPara 3";
    let config = ParseConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    assert_eq!(doc.content.len(), 3);
}

#[test]
fn test_parser_preserves_raw_source() {
    let parser = PlainTextHandler::new();
    let input = "Test input";
    let mut config = ParseConfig::default();
    config.preserve_raw_source = true;

    let doc = parser.parse(input, &config).expect("parse failed");
    assert!(doc.raw_source.is_some());
    assert_eq!(doc.raw_source.as_ref().unwrap(), input);
}

#[test]
fn test_parser_skips_whitespace() {
    let parser = PlainTextHandler::new();
    let input = "   \n\n  \n\n";
    let config = ParseConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    assert_eq!(doc.content.len(), 0);
}

#[test]
fn test_parser_trims_paragraphs() {
    let parser = PlainTextHandler::new();
    let input = "  leading and trailing  \n\n  second  ";
    let config = ParseConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    assert_eq!(doc.content.len(), 2);
}

#[test]
fn test_parser_format_identification() {
    let parser = PlainTextHandler::new();
    assert_eq!(parser.format(), SourceFormat::PlainText);
}

// ============================================================================
// Renderer Tests
// ============================================================================

#[test]
fn test_renderer_single_paragraph() {
    let renderer = PlainTextHandler::new();
    let doc = Document {
        source_format: SourceFormat::PlainText,
        meta: DocumentMeta::default(),
        content: vec![Block::Paragraph {
            content: vec![Inline::Text {
                content: "Hello world".to_string(),
            }],
            span: None,
        }],
        raw_source: None,
    };

    let output = renderer.render(&doc, &RenderConfig::default()).expect("render failed");
    assert_eq!(output, "Hello world");
}

#[test]
fn test_renderer_multiple_paragraphs() {
    let renderer = PlainTextHandler::new();
    let doc = Document {
        source_format: SourceFormat::PlainText,
        meta: DocumentMeta::default(),
        content: vec![
            Block::Paragraph {
                content: vec![Inline::Text {
                    content: "Para 1".to_string(),
                }],
                span: None,
            },
            Block::Paragraph {
                content: vec![Inline::Text {
                    content: "Para 2".to_string(),
                }],
                span: None,
            },
        ],
        raw_source: None,
    };

    let output = renderer.render(&doc, &RenderConfig::default()).expect("render failed");
    assert_eq!(output, "Para 1\n\nPara 2");
}

#[test]
fn test_renderer_empty_document() {
    let renderer = PlainTextHandler::new();
    let doc = Document {
        source_format: SourceFormat::PlainText,
        meta: DocumentMeta::default(),
        content: vec![],
        raw_source: None,
    };

    let output = renderer.render(&doc, &RenderConfig::default()).expect("render failed");
    assert_eq!(output, "");
}

#[test]
fn test_renderer_heading() {
    let renderer = PlainTextHandler::new();
    let doc = Document {
        source_format: SourceFormat::PlainText,
        meta: DocumentMeta::default(),
        content: vec![Block::Heading {
            level: 1,
            content: vec![Inline::Text {
                content: "Title".to_string(),
            }],
            span: None,
        }],
        raw_source: None,
    };

    let output = renderer.render(&doc, &RenderConfig::default()).expect("render failed");
    assert_eq!(output, "Title");
}

#[test]
fn test_renderer_format_identification() {
    let renderer = PlainTextHandler::new();
    assert_eq!(renderer.format(), SourceFormat::PlainText);
}

// ============================================================================
// Round-trip Tests
// ============================================================================

#[test]
fn test_roundtrip_simple() {
    let parser = PlainTextHandler::new();
    let renderer = PlainTextHandler::new();

    let input = "Hello world";
    let config = ParseConfig::default();
    let render_config = RenderConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    let output = renderer.render(&doc, &render_config).expect("render failed");

    assert_eq!(output, input);
}

#[test]
fn test_roundtrip_multiple_paragraphs() {
    let parser = PlainTextHandler::new();
    let renderer = PlainTextHandler::new();

    let input = "Para 1\n\nPara 2\n\nPara 3";
    let config = ParseConfig::default();
    let render_config = RenderConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    let output = renderer.render(&doc, &render_config).expect("render failed");

    assert_eq!(output, input);
}

// ============================================================================
// AST/Document Tests
// ============================================================================

#[test]
fn test_document_metadata_creation() {
    let meta = DocumentMeta {
        title: Some("Title".to_string()),
        authors: vec!["Author 1".to_string()],
        date: Some("2026-04-04".to_string()),
        language: Some("en".to_string()),
        custom: HashMap::new(),
    };

    assert_eq!(meta.title, Some("Title".to_string()));
    assert_eq!(meta.authors.len(), 1);
}

#[test]
fn test_document_clone() {
    let doc = Document {
        source_format: SourceFormat::PlainText,
        meta: DocumentMeta {
            title: Some("Title".to_string()),
            ..Default::default()
        },
        content: vec![Block::Paragraph {
            content: vec![Inline::Text {
                content: "Content".to_string(),
            }],
            span: None,
        }],
        raw_source: Some("Raw".to_string()),
    };

    let cloned = doc.clone();
    assert_eq!(doc.source_format, cloned.source_format);
    assert_eq!(doc.meta.title, cloned.meta.title);
    assert_eq!(doc.content.len(), cloned.content.len());
}

#[test]
fn test_source_format_equality() {
    let fmt1 = SourceFormat::PlainText;
    let fmt2 = SourceFormat::PlainText;
    assert_eq!(fmt1, fmt2);

    let fmt3 = SourceFormat::Markdown;
    assert_ne!(fmt1, fmt3);
}

#[test]
fn test_source_format_extensions() {
    assert_eq!(SourceFormat::PlainText.extension(), "txt");
    assert_eq!(SourceFormat::Markdown.extension(), "md");
    assert_eq!(SourceFormat::AsciiDoc.extension(), "adoc");
    assert_eq!(SourceFormat::Djot.extension(), "dj");
    assert_eq!(SourceFormat::OrgMode.extension(), "org");
    assert_eq!(SourceFormat::ReStructuredText.extension(), "rst");
    assert_eq!(SourceFormat::Typst.extension(), "typ");
}

#[test]
fn test_source_format_labels() {
    assert_eq!(SourceFormat::PlainText.label(), "TXT");
    assert_eq!(SourceFormat::Markdown.label(), "MD");
    assert_eq!(SourceFormat::AsciiDoc.label(), "ADOC");
}

#[test]
fn test_source_format_all_enumeration() {
    let all = SourceFormat::ALL;
    assert_eq!(all.len(), 7);
    assert!(all.contains(&SourceFormat::PlainText));
    assert!(all.contains(&SourceFormat::Markdown));
}

// ============================================================================
// Inline Block Tests
// ============================================================================

#[test]
fn test_inline_text_creation() {
    let inline = Inline::Text {
        content: "Test content".to_string(),
    };

    match inline {
        Inline::Text { content } => assert_eq!(content, "Test content"),
        _ => panic!("expected text inline"),
    }
}

#[test]
fn test_block_paragraph_creation() {
    let block = Block::Paragraph {
        content: vec![Inline::Text {
            content: "Content".to_string(),
        }],
        span: None,
    };

    match block {
        Block::Paragraph { content, .. } => assert_eq!(content.len(), 1),
        _ => panic!("expected paragraph block"),
    }
}

#[test]
fn test_block_heading_creation() {
    let block = Block::Heading {
        level: 2,
        content: vec![Inline::Text {
            content: "Heading".to_string(),
        }],
        span: None,
    };

    match block {
        Block::Heading { level, content, .. } => {
            assert_eq!(level, 2);
            assert_eq!(content.len(), 1);
        }
        _ => panic!("expected heading block"),
    }
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_parse_config_default() {
    let config = ParseConfig::default();
    assert!(!config.preserve_spans);
    assert!(!config.preserve_raw_source);
    assert!(config.front_matter_delimiter.is_none());
    assert!(config.format_options.is_empty());
}

#[test]
fn test_parse_config_customization() {
    let mut config = ParseConfig::default();
    config.preserve_spans = true;
    config.preserve_raw_source = true;
    config.front_matter_delimiter = Some("---".to_string());
    config.format_options.insert("key".to_string(), "value".to_string());

    assert!(config.preserve_spans);
    assert!(config.preserve_raw_source);
    assert_eq!(config.front_matter_delimiter, Some("---".to_string()));
    assert_eq!(config.format_options.get("key"), Some(&"value".to_string()));
}

#[test]
fn test_render_config_default() {
    let config = RenderConfig::default();
    assert_eq!(config.line_width, 80);
    assert_eq!(config.indent, "  ");
    assert!(!config.hard_breaks);
    assert!(config.format_options.is_empty());
}

#[test]
fn test_render_config_customization() {
    let mut config = RenderConfig::default();
    config.line_width = 120;
    config.indent = "\t".to_string();
    config.hard_breaks = true;
    config.format_options.insert("opt".to_string(), "val".to_string());

    assert_eq!(config.line_width, 120);
    assert_eq!(config.indent, "\t");
    assert!(config.hard_breaks);
    assert_eq!(config.format_options.get("opt"), Some(&"val".to_string()));
}

// ============================================================================
// FormatHandler Tests
// ============================================================================

#[test]
fn test_format_handler_features() {
    let handler = PlainTextHandler::new();
    assert!(!handler.supports_feature("bold"));
    assert!(!handler.supports_feature("italic"));
    assert_eq!(handler.supported_features().len(), 0);
}

// ============================================================================
// Metadata Value Tests
// ============================================================================

#[test]
fn test_metavalue_string() {
    let val = MetaValue::String("test".to_string());
    match val {
        MetaValue::String(s) => assert_eq!(s, "test"),
        _ => panic!("expected string"),
    }
}

#[test]
fn test_metavalue_bool() {
    let val = MetaValue::Bool(true);
    match val {
        MetaValue::Bool(b) => assert!(b),
        _ => panic!("expected bool"),
    }
}

#[test]
fn test_metavalue_integer() {
    let val = MetaValue::Integer(42);
    match val {
        MetaValue::Integer(i) => assert_eq!(i, 42),
        _ => panic!("expected integer"),
    }
}

#[test]
fn test_metavalue_float() {
    let val = MetaValue::Float(3.14);
    match val {
        MetaValue::Float(f) => assert!((f - 3.14).abs() < 0.01),
        _ => panic!("expected float"),
    }
}

#[test]
fn test_metavalue_list() {
    let val = MetaValue::List(vec![
        MetaValue::String("a".to_string()),
        MetaValue::String("b".to_string()),
    ]);
    match val {
        MetaValue::List(items) => assert_eq!(items.len(), 2),
        _ => panic!("expected list"),
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_parse_never_panics_on_input() {
    let parser = PlainTextHandler::new();
    let malformed_inputs = vec![
        "",
        " ",
        "\n",
        "\0",
        "A".repeat(100_000).as_str(),
        "\u{FFFD}",
    ];

    for input in malformed_inputs {
        let result = parser.parse(input, &ParseConfig::default());
        // Should either succeed or return an error, never panic
        let _ = result.is_ok();
    }
}

#[test]
fn test_render_never_panics_on_document() {
    let renderer = PlainTextHandler::new();
    let docs = vec![
        Document {
            source_format: SourceFormat::PlainText,
            meta: DocumentMeta::default(),
            content: vec![],
            raw_source: None,
        },
        Document {
            source_format: SourceFormat::PlainText,
            meta: DocumentMeta {
                title: Some("A".repeat(100_000)),
                ..Default::default()
            },
            content: vec![Block::Paragraph {
                content: vec![Inline::Text {
                    content: "A".repeat(100_000),
                }],
                span: None,
            }],
            raw_source: None,
        },
    ];

    for doc in docs {
        let result = renderer.render(&doc, &RenderConfig::default());
        // Should either succeed or return an error, never panic
        let _ = result.is_ok();
    }
}
