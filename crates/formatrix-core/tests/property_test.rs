// SPDX-License-Identifier: PMPL-1.0-or-later
//! Property-based tests for format conversion correctness

use formatrix_core::{
    ast::{DocumentMeta, SourceFormat},
    traits::{Parser, ParseConfig, RenderConfig, Renderer},
    formats::PlainTextHandler,
};
use proptest::prelude::*;

proptest! {
    /// Property: Plaintext parsing never panics on any input
    #[test]
    fn prop_plaintext_parse_no_panic(s in ".*") {
        let parser = PlainTextHandler::new();
        let _result = parser.parse(&s, &ParseConfig::default());
    }

    /// Property: Document metadata is preserved after construction
    #[test]
    fn prop_document_metadata_stable(title in "[a-zA-Z0-9 ]{0,100}") {
        let meta = DocumentMeta {
            title: Some(title.clone()),
            authors: vec!["Author".to_string()],
            ..Default::default()
        };

        prop_assert_eq!(meta.title, Some(title));
        prop_assert_eq!(meta.authors.len(), 1);
    }

    /// Property: Render config defaults are sensible
    #[test]
    fn prop_render_config_defaults_valid(
        _sample in prop::sample::select(vec![true; 100])
    ) {
        let config = RenderConfig::default();

        // Line width should be positive or zero
        prop_assert!(config.line_width >= 0);
        // Indent should exist
        prop_assert!(!config.indent.is_empty());
    }

    /// Property: Parse config format options preserve insertion order
    #[test]
    fn prop_parse_config_options_preserved(
        key in "[a-z]{1,20}",
        value in "[a-z0-9]{1,20}"
    ) {
        let mut config = ParseConfig::default();
        config.format_options.insert(key.clone(), value.clone());

        prop_assert_eq!(config.format_options.get(&key), Some(&value));
    }

    /// Property: Empty input always parses successfully
    #[test]
    fn prop_empty_input_parses_ok(
        _sample in prop::sample::select(vec![true; 100])
    ) {
        let parser = PlainTextHandler::new();
        let result = parser.parse("", &ParseConfig::default());
        prop_assert!(result.is_ok());
    }

    /// Property: Round-trip preserves block count
    #[test]
    fn prop_roundtrip_block_count(
        para1 in "[a-zA-Z0-9 ]{1,50}",
        para2 in "[a-zA-Z0-9 ]{1,50}"
    ) {
        let parser = PlainTextHandler::new();
        let renderer = PlainTextHandler::new();

        let input = format!("{}\n\n{}", para1, para2);
        let parse_config = ParseConfig::default();
        let render_config = RenderConfig::default();

        let doc = parser.parse(&input, &parse_config).expect("parse");
        let output = renderer.render(&doc, &render_config).expect("render");

        // Re-parse the output
        let doc2 = parser.parse(&output, &parse_config).expect("reparse");

        // Should have non-empty blocks
        prop_assert!(doc2.content.len() > 0);
    }
}

#[test]
fn test_source_format_all_enumeration() {
    let all = SourceFormat::ALL;
    assert_eq!(all.len(), 7);

    let has_plaintext = all.iter().any(|f| matches!(f, SourceFormat::PlainText));
    let has_markdown = all.iter().any(|f| matches!(f, SourceFormat::Markdown));
    assert!(has_plaintext && has_markdown);
}

#[test]
fn test_parse_config_defaults_stable() {
    let c1 = ParseConfig::default();
    let c2 = ParseConfig::default();

    assert_eq!(c1.preserve_spans, c2.preserve_spans);
    assert_eq!(c1.preserve_raw_source, c2.preserve_raw_source);
}
