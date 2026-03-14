// SPDX-License-Identifier: PMPL-1.0-or-later
//! Plain text format handler

use crate::ast::{Block, Document, DocumentMeta, Inline, SourceFormat};
use crate::traits::{FormatHandler, ParseConfig, Parser, RenderConfig, Renderer, Result};

/// Plain text format handler
pub struct PlainTextHandler;

impl PlainTextHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlainTextHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for PlainTextHandler {
    fn format(&self) -> SourceFormat {
        SourceFormat::PlainText
    }

    fn parse(&self, input: &str, config: &ParseConfig) -> Result<Document> {
        // Split into paragraphs on blank lines
        let paragraphs: Vec<Block> = input
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .map(|p| Block::Paragraph {
                content: vec![Inline::Text {
                    content: p.trim().to_string(),
                }],
                span: None,
            })
            .collect();

        Ok(Document {
            source_format: SourceFormat::PlainText,
            meta: DocumentMeta::default(),
            content: paragraphs,
            raw_source: if config.preserve_raw_source {
                Some(input.to_string())
            } else {
                None
            },
        })
    }
}

impl Renderer for PlainTextHandler {
    fn format(&self) -> SourceFormat {
        SourceFormat::PlainText
    }

    fn render(&self, doc: &Document, _config: &RenderConfig) -> Result<String> {
        let mut output = String::new();

        for (i, block) in doc.content.iter().enumerate() {
            if i > 0 {
                output.push_str("\n\n");
            }
            render_block(&mut output, block);
        }

        Ok(output)
    }
}

fn render_block(output: &mut String, block: &Block) {
    match block {
        Block::Paragraph { content, .. } => {
            for inline in content {
                render_inline(output, inline);
            }
        }
        Block::Heading { content, .. } => {
            for inline in content {
                render_inline(output, inline);
            }
        }
        Block::CodeBlock { content, .. } => {
            output.push_str(content);
        }
        Block::BlockQuote { content, .. } => {
            for block in content {
                render_block(output, block);
            }
        }
        Block::List { items, .. } => {
            for item in items {
                for block in &item.content {
                    render_block(output, block);
                }
            }
        }
        Block::Raw { content, .. } => {
            output.push_str(content);
        }
        _ => {}
    }
}

fn render_inline(output: &mut String, inline: &Inline) {
    match inline {
        Inline::Text { content } => output.push_str(content),
        Inline::Emphasis { content } => {
            for i in content {
                render_inline(output, i);
            }
        }
        Inline::Strong { content } => {
            for i in content {
                render_inline(output, i);
            }
        }
        Inline::Code { content, .. } => output.push_str(content),
        Inline::Link { content, .. } => {
            for i in content {
                render_inline(output, i);
            }
        }
        Inline::LineBreak => output.push('\n'),
        Inline::SoftBreak => output.push(' '),
        _ => {}
    }
}

impl FormatHandler for PlainTextHandler {
    fn supports_feature(&self, _feature: &str) -> bool {
        false // Plain text doesn't support any special features
    }

    fn supported_features(&self) -> &[&str] {
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let handler = PlainTextHandler::new();
        let doc = handler
            .parse("Hello world\n\nSecond paragraph", &ParseConfig::default())
            .unwrap();

        assert_eq!(doc.content.len(), 2);
    }

    #[test]
    fn test_roundtrip() {
        let handler = PlainTextHandler::new();
        let input = "Hello world\n\nSecond paragraph";
        let doc = handler.parse(input, &ParseConfig::default()).unwrap();
        let output = handler.render(&doc, &RenderConfig::default()).unwrap();

        assert_eq!(output, input);
    }
}
