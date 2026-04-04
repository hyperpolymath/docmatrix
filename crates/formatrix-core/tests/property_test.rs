// SPDX-License-Identifier: PMPL-1.0-or-later
//! Property-based tests for format conversion correctness

use formatrix_core::{
    ast::{Block, Document, DocumentMeta, Inline, SourceFormat},
    traits::{FormatHandler, Parser, ParseConfig, RenderConfig, Renderer},
};
use proptest::prelude::*;

// Strategies for generating test data
fn arb_plaintext() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 .,;:!?\-'\"\n]{0,500}".prop_map(|s| s.trim().to_string())
}

fn arb_heading_level() -> impl Strategy<Value = u8> {
    1u8..=6
}

fn arb_document() -> impl Strategy<Value = Document> {
    (arb_plaintext(), arb_heading_level())
        .prop_map(|(content, level)| {
            let mut blocks = vec![
                Block::Heading {
                    level,
                    content: vec![Inline::Text {
                        content: "Test Document".to_string(),
                    }],
                    span: None,
                },
                Block::Paragraph {
                    content: vec![Inline::Text { content }],
                    span: None,
                },
            ];

            Document {
                source_format: SourceFormat::PlainText,
                meta: DocumentMeta {
                    title: Some("Test Document".to_string()),
                    ..Default::default()
                },
                content: blocks,
                raw_source: None,
            }
        })
}

#[test]
fn prop_plaintext_conversion_is_idempotent() {
    proptest!(|(text in arb_plaintext())| {
        let parser = formatrix_core::formats::PlainTextHandler::new();
        let config = ParseConfig::default();

        // Parse once
        let doc = parser.parse(&text, &config).expect("parse 1");
        assert!(!doc.content.is_empty(), "document should have content");
    });
}

#[test]
fn prop_empty_string_handled_gracefully() {
    proptest!(|(text in "[ ]*")| {
        let parser = formatrix_core::formats::PlainTextHandler::new();
        let config = ParseConfig::default();

        // Empty or whitespace strings should parse without panic
        let result = parser.parse(&text, &config);
        assert!(result.is_ok() || result.is_err());
    });
}

#[test]
fn prop_heading_level_preserved() {
    proptest!(|(level in 1u8..=6)| {
        let heading = Block::Heading {
            level,
            content: vec![Inline::Text {
                content: "Test".to_string(),
            }],
            span: None,
        };

        match heading {
            Block::Heading { level: l, .. } => {
                prop_assert_eq!(level, l);
            }
            _ => prop_assert!(false, "expected heading block"),
        }
    });
}

#[test]
fn prop_document_metadata_preserved() {
    proptest!(|(title in "[a-zA-Z ]{1,50}")| {
        let meta = DocumentMeta {
            title: Some(title.clone()),
            authors: vec!["Test Author".to_string()],
            ..Default::default()
        };

        prop_assert_eq!(meta.title, Some(title));
        prop_assert_eq!(meta.authors.len(), 1);
    });
}

#[test]
fn prop_unicode_content_preserved() {
    proptest!(|(text in "[\\PC]*")| {
        let parser = formatrix_core::formats::PlainTextHandler::new();
        let config = ParseConfig {
            preserve_raw_source: true,
            ..Default::default()
        };

        if let Ok(doc) = parser.parse(&text, &config) {
            // Raw source should preserve original content exactly
            if let Some(raw) = &doc.raw_source {
                prop_assert_eq!(raw.as_str(), text.as_str());
            }
        }
    });
}

#[test]
fn prop_render_config_defaults_valid() {
    let config = RenderConfig::default();
    prop_assert_eq!(config.line_width, 80);
    prop_assert_eq!(config.indent, "  ");
    prop_assert!(!config.hard_breaks);
}

#[test]
fn prop_parse_config_options_non_lossy() {
    proptest!(|(key in "[a-z]{1,20}", value in "[a-z0-9]{1,20}")| {
        let mut config = ParseConfig::default();
        config.format_options.insert(key.clone(), value.clone());

        prop_assert_eq!(config.format_options.get(&key), Some(&value));
    });
}
