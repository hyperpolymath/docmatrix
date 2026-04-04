// SPDX-License-Identifier: PMPL-1.0-or-later
//! End-to-end format conversion tests

use formatrix_core::{
    ast::{Block, Document, DocumentMeta, Inline, SourceFormat},
    traits::{FormatHandler, Parser, ParseConfig, RenderConfig, Renderer},
    formats::PlainTextHandler,
};

/// Test basic plaintext parsing
#[test]
fn test_plaintext_parse_simple_paragraph() {
    let parser = PlainTextHandler::new();
    let input = "This is a test paragraph.";
    let config = ParseConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    assert_eq!(doc.source_format, SourceFormat::PlainText);
    assert_eq!(doc.content.len(), 1);

    match &doc.content[0] {
        Block::Paragraph { content, .. } => {
            assert_eq!(content.len(), 1);
            match &content[0] {
                Inline::Text { content } => {
                    assert_eq!(content, "This is a test paragraph.");
                }
                _ => panic!("expected text inline"),
            }
        }
        _ => panic!("expected paragraph block"),
    }
}

/// Test plaintext with multiple paragraphs
#[test]
fn test_plaintext_parse_multiple_paragraphs() {
    let parser = PlainTextHandler::new();
    let input = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
    let config = ParseConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    assert_eq!(doc.content.len(), 3, "should have 3 paragraphs");
}

/// Test plaintext with preserved raw source
#[test]
fn test_plaintext_preserve_raw_source() {
    let parser = PlainTextHandler::new();
    let input = "Test content with\nmultiple lines.";
    let mut config = ParseConfig::default();
    config.preserve_raw_source = true;

    let doc = parser.parse(input, &config).expect("parse failed");
    assert!(doc.raw_source.is_some(), "raw_source should be preserved");
    assert_eq!(doc.raw_source.as_ref().unwrap(), input);
}

/// Test plaintext render from AST
#[test]
fn test_plaintext_render_document() {
    let parser = PlainTextHandler::new();
    let renderer = PlainTextHandler::new();

    let doc = Document {
        source_format: SourceFormat::PlainText,
        meta: DocumentMeta {
            title: Some("Test Document".to_string()),
            ..Default::default()
        },
        content: vec![
            Block::Paragraph {
                content: vec![Inline::Text {
                    content: "Hello, world!".to_string(),
                }],
                span: None,
            },
        ],
        raw_source: None,
    };

    let config = RenderConfig::default();
    let result = renderer.render(&doc, &config).expect("render failed");
    assert!(!result.is_empty(), "rendered output should not be empty");
}

/// Test empty document handling
#[test]
fn test_plaintext_empty_document() {
    let parser = PlainTextHandler::new();
    let input = "";
    let config = ParseConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    assert_eq!(doc.content.len(), 0, "empty input should produce empty content");
}

/// Test document with only whitespace
#[test]
fn test_plaintext_whitespace_only() {
    let parser = PlainTextHandler::new();
    let input = "   \n\n  \t  \n";
    let config = ParseConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    assert_eq!(doc.content.len(), 0, "whitespace-only input should produce no blocks");
}

/// Test round-trip: parse then render
#[test]
fn test_plaintext_round_trip() {
    let parser = PlainTextHandler::new();
    let renderer = PlainTextHandler::new();

    let input = "Test paragraph one.\n\nTest paragraph two.";
    let parse_config = ParseConfig::default();
    let render_config = RenderConfig::default();

    // Parse
    let doc = parser.parse(input, &parse_config).expect("parse failed");

    // Render
    let output = renderer.render(&doc, &render_config).expect("render failed");

    // Both should be valid (content-wise equivalent)
    assert!(!output.is_empty(), "round-trip output should not be empty");

    // Re-parse the output
    let doc2 = parser.parse(&output, &parse_config).expect("re-parse failed");
    assert_eq!(doc.content.len(), doc2.content.len(), "round-trip block count mismatch");
}

/// Test parser format identification
#[test]
fn test_parser_format_identification() {
    let parser = PlainTextHandler::new();
    assert_eq!(parser.format(), SourceFormat::PlainText);
}

/// Test renderer format identification
#[test]
fn test_renderer_format_identification() {
    let renderer = PlainTextHandler::new();
    assert_eq!(renderer.format(), SourceFormat::PlainText);
}

/// Test document metadata handling
#[test]
fn test_document_metadata() {
    let doc = Document {
        source_format: SourceFormat::PlainText,
        meta: DocumentMeta {
            title: Some("My Document".to_string()),
            authors: vec!["Author One".to_string(), "Author Two".to_string()],
            date: Some("2026-04-04".to_string()),
            language: Some("en".to_string()),
            ..Default::default()
        },
        content: vec![],
        raw_source: None,
    };

    assert_eq!(doc.meta.title, Some("My Document".to_string()));
    assert_eq!(doc.meta.authors.len(), 2);
    assert_eq!(doc.meta.date, Some("2026-04-04".to_string()));
    assert_eq!(doc.meta.language, Some("en".to_string()));
}

/// Test source format extension methods
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

/// Test source format labels
#[test]
fn test_source_format_labels() {
    assert_eq!(SourceFormat::PlainText.label(), "TXT");
    assert_eq!(SourceFormat::Markdown.label(), "MD");
    assert_eq!(SourceFormat::AsciiDoc.label(), "ADOC");
    assert_eq!(SourceFormat::Djot.label(), "DJOT");
    assert_eq!(SourceFormat::OrgMode.label(), "ORG");
    assert_eq!(SourceFormat::ReStructuredText.label(), "RST");
    assert_eq!(SourceFormat::Typst.label(), "TYP");
}

/// Test all formats are enumerated
#[test]
fn test_source_format_all() {
    let all_formats = SourceFormat::ALL;
    assert_eq!(all_formats.len(), 7);
    assert!(all_formats.contains(&SourceFormat::PlainText));
    assert!(all_formats.contains(&SourceFormat::Markdown));
    assert!(all_formats.contains(&SourceFormat::AsciiDoc));
}

/// Test document clone and copy semantics
#[test]
fn test_document_clone() {
    let original = Document {
        source_format: SourceFormat::PlainText,
        meta: DocumentMeta {
            title: Some("Original".to_string()),
            ..Default::default()
        },
        content: vec![
            Block::Paragraph {
                content: vec![Inline::Text {
                    content: "Content".to_string(),
                }],
                span: None,
            },
        ],
        raw_source: Some("Raw".to_string()),
    };

    let cloned = original.clone();
    assert_eq!(original.source_format, cloned.source_format);
    assert_eq!(original.meta.title, cloned.meta.title);
    assert_eq!(original.content.len(), cloned.content.len());
}

/// Test configuration immutability
#[test]
fn test_parse_config_immutability() {
    let config1 = ParseConfig::default();
    let config2 = ParseConfig::default();

    assert_eq!(config1.preserve_spans, config2.preserve_spans);
    assert_eq!(config1.preserve_raw_source, config2.preserve_raw_source);
}

/// Test render config customization
#[test]
fn test_render_config_customization() {
    let mut config = RenderConfig::default();
    config.line_width = 120;
    config.indent = "\t".to_string();
    config.hard_breaks = true;

    assert_eq!(config.line_width, 120);
    assert_eq!(config.indent, "\t");
    assert!(config.hard_breaks);
}
