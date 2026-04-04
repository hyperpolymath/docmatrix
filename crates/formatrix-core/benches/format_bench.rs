// SPDX-License-Identifier: PMPL-1.0-or-later
//! Benchmark tests for format conversion performance

use formatrix_core::{
    ast::{Block, Document, DocumentMeta, Inline, SourceFormat},
    traits::{Parser, ParseConfig, RenderConfig, Renderer},
    formats::PlainTextHandler,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Benchmark plaintext parsing of small documents
fn bench_parse_small_plaintext(c: &mut Criterion) {
    c.bench_function("parse_small_plaintext_100bytes", |b| {
        b.iter(|| {
            let parser = PlainTextHandler::new();
            let input = black_box("Lorem ipsum dolor sit amet.\n\nConsectetur adipiscing elit.");
            let config = ParseConfig::default();
            parser.parse(input, &config)
        })
    });
}

/// Benchmark plaintext parsing of medium documents
fn bench_parse_medium_plaintext(c: &mut Criterion) {
    c.bench_function("parse_medium_plaintext_10kb", |b| {
        b.iter(|| {
            let parser = PlainTextHandler::new();
            let input = black_box(&"Lorem ipsum dolor sit amet.\n\n".repeat(500));
            let config = ParseConfig::default();
            parser.parse(input, &config)
        })
    });
}

/// Benchmark plaintext parsing of large documents
fn bench_parse_large_plaintext(c: &mut Criterion) {
    c.bench_function("parse_large_plaintext_100kb", |b| {
        b.iter(|| {
            let parser = PlainTextHandler::new();
            let input = black_box(&"Lorem ipsum dolor sit amet.\n\n".repeat(5000));
            let config = ParseConfig::default();
            parser.parse(input, &config)
        })
    });
}

/// Benchmark plaintext rendering
fn bench_render_plaintext(c: &mut Criterion) {
    c.bench_function("render_plaintext_paragraph", |b| {
        let doc = Document {
            source_format: SourceFormat::PlainText,
            meta: DocumentMeta::default(),
            content: vec![
                Block::Paragraph {
                    content: vec![Inline::Text {
                        content: "Test paragraph content.".to_string(),
                    }],
                    span: None,
                },
            ],
            raw_source: None,
        };

        b.iter(|| {
            let renderer = PlainTextHandler::new();
            let config = RenderConfig::default();
            renderer.render(black_box(&doc), &config)
        })
    });
}

/// Benchmark round-trip conversion (parse + render)
fn bench_round_trip(c: &mut Criterion) {
    c.bench_function("round_trip_plaintext", |b| {
        b.iter(|| {
            let parser = PlainTextHandler::new();
            let renderer = PlainTextHandler::new();
            let input = black_box("Test paragraph one.\n\nTest paragraph two.");

            let parse_config = ParseConfig::default();
            let render_config = RenderConfig::default();

            let doc = parser.parse(input, &parse_config).unwrap();
            renderer.render(&doc, &render_config)
        })
    });
}

/// Benchmark parsing with raw source preservation
fn bench_parse_with_source_preservation(c: &mut Criterion) {
    c.bench_function("parse_with_source_preservation", |b| {
        b.iter(|| {
            let parser = PlainTextHandler::new();
            let input = black_box("Test content with preservation.\n\nAnother paragraph.");
            let mut config = ParseConfig::default();
            config.preserve_raw_source = true;

            parser.parse(input, &config)
        })
    });
}

/// Benchmark parsing with span preservation
fn bench_parse_with_span_preservation(c: &mut Criterion) {
    c.bench_function("parse_with_span_preservation", |b| {
        b.iter(|| {
            let parser = PlainTextHandler::new();
            let input = black_box("Test content with spans.\n\nAnother paragraph.");
            let mut config = ParseConfig::default();
            config.preserve_spans = true;

            parser.parse(input, &config)
        })
    });
}

/// Benchmark document creation
fn bench_document_creation(c: &mut Criterion) {
    c.bench_function("create_document_with_metadata", |b| {
        b.iter(|| {
            Document {
                source_format: SourceFormat::PlainText,
                meta: DocumentMeta {
                    title: Some("Test Title".to_string()),
                    authors: vec!["Author".to_string()],
                    date: Some("2026-04-04".to_string()),
                    language: Some("en".to_string()),
                    ..Default::default()
                },
                content: vec![
                    Block::Paragraph {
                        content: vec![Inline::Text {
                            content: black_box("Content".to_string()),
                        }],
                        span: None,
                    },
                ],
                raw_source: None,
            }
        })
    });
}

/// Benchmark multiple document parsing in sequence
fn bench_batch_parsing(c: &mut Criterion) {
    c.bench_function("batch_parse_10_documents", |b| {
        b.iter(|| {
            let parser = PlainTextHandler::new();
            let config = ParseConfig::default();

            for i in 0..10 {
                let input = black_box(&format!("Document {}.\n\nContent.", i));
                let _ = parser.parse(input, &config);
            }
        })
    });
}

criterion_group!(
    benches,
    bench_parse_small_plaintext,
    bench_parse_medium_plaintext,
    bench_parse_large_plaintext,
    bench_render_plaintext,
    bench_round_trip,
    bench_parse_with_source_preservation,
    bench_parse_with_span_preservation,
    bench_document_creation,
    bench_batch_parsing,
);

criterion_main!(benches);
