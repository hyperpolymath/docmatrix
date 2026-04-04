// SPDX-License-Identifier: PMPL-1.0-or-later
//! Aspect and cross-cutting concern tests

use formatrix_core::{
    ast::{Document, DocumentMeta, SourceFormat},
    traits::{Parser, ParseConfig},
    formats::PlainTextHandler,
};

/// Test handling of oversized documents (1MB+)
#[test]
fn test_large_document_handling() {
    let parser = PlainTextHandler::new();

    // Create a 1MB document
    let large_content = "A".repeat(1_000_000);
    let config = ParseConfig::default();

    let result = parser.parse(&large_content, &config);
    assert!(result.is_ok(), "should handle large documents without panic");
}

/// Test handling of extremely deep nesting
#[test]
fn test_deeply_nested_elements() {
    let parser = PlainTextHandler::new();

    // Create deeply nested paragraph-like structure with newlines
    let mut nested = String::new();
    for i in 0..500 {
        nested.push_str(&format!("Level {}\n\n", i));
    }

    let config = ParseConfig::default();
    let result = parser.parse(&nested, &config);
    assert!(result.is_ok(), "should handle deep nesting without stack overflow");
}

/// Test handling of null bytes (safety aspect)
#[test]
fn test_null_byte_handling() {
    let parser = PlainTextHandler::new();

    // Create input with null bytes
    let input_with_nulls = "Before\0After";
    let config = ParseConfig::default();

    let result = parser.parse(input_with_nulls, &config);
    // Should either succeed or fail gracefully, not panic
    assert!(result.is_ok() || result.is_err());
}

/// Test handling of various line endings
#[test]
fn test_mixed_line_endings() {
    let parser = PlainTextHandler::new();

    // Test with mixed CRLF and LF
    let input = "Line 1\r\nLine 2\nLine 3\r\n";
    let config = ParseConfig::default();

    let result = parser.parse(input, &config);
    assert!(result.is_ok(), "should handle mixed line endings");
}

/// Test Unicode preservation (CJK, RTL, emoji)
#[test]
fn test_unicode_cjk_preservation() {
    let parser = PlainTextHandler::new();

    let input = "日本語テキスト漢字";
    let config = ParseConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    assert!(!doc.content.is_empty(), "CJK text should parse successfully");
}

/// Test Unicode emoji preservation
#[test]
fn test_unicode_emoji_preservation() {
    let parser = PlainTextHandler::new();

    let input = "Hello 👋 World 🌍 Test 🧪";
    let config = ParseConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    assert!(!doc.content.is_empty(), "emoji should parse successfully");
}

/// Test right-to-left text preservation
#[test]
fn test_unicode_rtl_preservation() {
    let parser = PlainTextHandler::new();

    let input = "Hello עברית Arabic العربية";
    let config = ParseConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    assert!(!doc.content.is_empty(), "RTL text should parse successfully");
}

/// Test zero-width characters
#[test]
fn test_zero_width_characters() {
    let parser = PlainTextHandler::new();

    // Include zero-width space, joiner, etc.
    let input = "Text\u{200B}with\u{200C}zero\u{200D}width";
    let config = ParseConfig::default();

    let result = parser.parse(input, &config);
    assert!(result.is_ok(), "should handle zero-width characters");
}

/// Test empty document edge case
#[test]
fn test_empty_document_no_panic() {
    let parser = PlainTextHandler::new();

    let input = "";
    let config = ParseConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    assert_eq!(doc.content.len(), 0, "empty input should produce empty content");
}

/// Test single whitespace character
#[test]
fn test_single_whitespace() {
    let parser = PlainTextHandler::new();

    for ws in &[" ", "\n", "\t", "\r"] {
        let config = ParseConfig::default();
        let result = parser.parse(ws, &config);
        assert!(result.is_ok(), "single whitespace should parse");
    }
}

/// Test very long single line (no breaks)
#[test]
fn test_very_long_line() {
    let parser = PlainTextHandler::new();

    let input = "a".repeat(100_000);
    let config = ParseConfig::default();

    let result = parser.parse(&input, &config);
    assert!(result.is_ok(), "should handle very long lines");
}

/// Test special characters (symbols, punctuation)
#[test]
fn test_special_characters() {
    let parser = PlainTextHandler::new();

    let input = "!@#$%^&*()_+-={}|:\";<>?,./~`";
    let config = ParseConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    assert!(!doc.content.is_empty(), "special characters should parse");
}

/// Test consecutive blank lines
#[test]
fn test_consecutive_blank_lines() {
    let parser = PlainTextHandler::new();

    let input = "Para1\n\n\n\n\n\nPara2";
    let config = ParseConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    // Should handle multiple blank lines gracefully
    assert!(doc.content.len() <= 2, "consecutive blanks should not create extra blocks");
}

/// Test document with only punctuation
#[test]
fn test_punctuation_only() {
    let parser = PlainTextHandler::new();

    let input = "...!!!???---...";
    let config = ParseConfig::default();

    let doc = parser.parse(input, &config).expect("parse failed");
    assert!(!doc.content.is_empty(), "punctuation should parse");
}

/// Test memory safety with repeated parsing
#[test]
fn test_repeated_parsing_memory_safety() {
    let parser = PlainTextHandler::new();
    let input = "Test content for repeated parsing.";
    let config = ParseConfig::default();

    // Parse the same content multiple times
    for _ in 0..1000 {
        let result = parser.parse(input, &config);
        assert!(result.is_ok(), "repeated parsing should not corrupt memory");
    }
}

/// Test format version stability
#[test]
fn test_source_format_stability() {
    let formats = SourceFormat::ALL;

    // Formats should be stable and not change between calls
    let formats2 = SourceFormat::ALL;
    assert_eq!(formats, formats2);
}

/// Test document metadata bounds
#[test]
fn test_document_metadata_large_values() {
    let large_title = "A".repeat(100_000);
    let meta = DocumentMeta {
        title: Some(large_title.clone()),
        authors: (0..1000)
            .map(|i| format!("Author {}", i))
            .collect(),
        ..Default::default()
    };

    assert_eq!(meta.title.as_ref().unwrap().len(), 100_000);
    assert_eq!(meta.authors.len(), 1000);
}

/// Test parse config with large option maps
#[test]
fn test_parse_config_large_options() {
    let mut config = ParseConfig::default();

    // Add many options
    for i in 0..1000 {
        config.format_options.insert(
            format!("option_{}", i),
            format!("value_{}", i),
        );
    }

    assert_eq!(config.format_options.len(), 1000);
}

/// Test concurrent parsing safety (single-threaded simulation)
#[test]
fn test_parsing_thread_safety_simulate() {
    let parser1 = PlainTextHandler::new();
    let parser2 = PlainTextHandler::new();

    let input = "Concurrent test content";
    let config = ParseConfig::default();

    // Simulate concurrent access to same parser type
    let result1 = parser1.parse(input, &config);
    let result2 = parser2.parse(input, &config);

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}
