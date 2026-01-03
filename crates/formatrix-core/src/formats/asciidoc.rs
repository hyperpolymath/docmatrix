// SPDX-License-Identifier: AGPL-3.0-or-later
//! AsciiDoc format handler using asciidoc-parser
//! FD-S01: SHOULD requirement

use crate::ast::{
    Block, Document, DocumentMeta, Inline, ListKind, SourceFormat,
};
use crate::traits::{FormatHandler, ParseConfig, Parser as ParserTrait, RenderConfig, Renderer, Result};
use asciidoc_parser::{Document as AdocDocument, Parser as AdocParser, blocks::IsBlock};

/// AsciiDoc format handler
pub struct AsciidocHandler;

impl AsciidocHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AsciidocHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserTrait for AsciidocHandler {
    fn format(&self) -> SourceFormat {
        SourceFormat::AsciiDoc
    }

    fn parse(&self, input: &str, config: &ParseConfig) -> Result<Document> {
        let mut parser = AdocParser::default();
        let adoc_doc = parser.parse(input);

        // Extract title from header
        let title = adoc_doc.header().title().map(|s| s.to_string());

        // Convert blocks
        let content = convert_blocks(&adoc_doc);

        Ok(Document {
            source_format: SourceFormat::AsciiDoc,
            meta: DocumentMeta {
                title,
                ..Default::default()
            },
            content,
            raw_source: if config.preserve_raw_source {
                Some(input.to_string())
            } else {
                None
            },
        })
    }
}

/// Convert asciidoc-parser blocks to our AST
fn convert_blocks(doc: &AdocDocument<'_>) -> Vec<Block> {
    let mut blocks = Vec::new();

    for block in doc.nested_blocks() {
        if let Some(converted) = convert_block(block) {
            blocks.push(converted);
        }
    }

    blocks
}

/// Convert a single asciidoc-parser block to our AST
fn convert_block(block: &asciidoc_parser::blocks::Block<'_>) -> Option<Block> {
    use asciidoc_parser::blocks::Block as AdocBlock;

    match block {
        AdocBlock::Simple(simple) => {
            let content = simple.content().rendered();
            Some(Block::Paragraph {
                content: parse_inline_content(content),
                span: None,
            })
        }

        AdocBlock::Section(section) => {
            // Section level is 1-indexed in AsciiDoc
            let level = section.level() as u8;
            let title_text = section.section_title().to_string();

            // Create heading
            Some(Block::Heading {
                level,
                content: vec![Inline::Text { content: title_text }],
                id: None,
                span: None,
            })
        }

        AdocBlock::RawDelimited(raw) => {
            let content = raw.content().rendered().to_string();
            let context = raw.raw_context();

            match context.as_ref() {
                "listing" | "source" => {
                    // Try to get language from attributes
                    let language = raw.attrlist()
                        .and_then(|a| a.named_or_positional_attribute("language", 1))
                        .map(|attr| attr.value().to_string());

                    Some(Block::CodeBlock {
                        language,
                        content,
                        line_numbers: false,
                        highlight_lines: Vec::new(),
                        span: None,
                    })
                }
                "literal" => {
                    Some(Block::Raw {
                        format: SourceFormat::AsciiDoc,
                        content,
                        span: None,
                    })
                }
                "pass" | "passthrough" => {
                    Some(Block::Raw {
                        format: SourceFormat::AsciiDoc,
                        content,
                        span: None,
                    })
                }
                _ => {
                    // Comment blocks are skipped
                    None
                }
            }
        }

        AdocBlock::CompoundDelimited(compound) => {
            let context = compound.raw_context();

            match context.as_ref() {
                "quote" | "verse" => {
                    let mut inner_blocks = Vec::new();
                    for child in compound.nested_blocks() {
                        if let Some(converted) = convert_block(child) {
                            inner_blocks.push(converted);
                        }
                    }
                    Some(Block::BlockQuote {
                        content: inner_blocks,
                        attribution: None,
                        admonition: None,
                        span: None,
                    })
                }
                "sidebar" => {
                    let mut inner_blocks = Vec::new();
                    for child in compound.nested_blocks() {
                        if let Some(converted) = convert_block(child) {
                            inner_blocks.push(converted);
                        }
                    }
                    Some(Block::Container {
                        id: None,
                        classes: vec!["sidebar".to_string()],
                        attributes: std::collections::HashMap::new(),
                        content: inner_blocks,
                        span: None,
                    })
                }
                _ => {
                    // Other compound blocks - convert nested
                    let mut inner_blocks = Vec::new();
                    for child in compound.nested_blocks() {
                        if let Some(converted) = convert_block(child) {
                            inner_blocks.push(converted);
                        }
                    }
                    if !inner_blocks.is_empty() {
                        Some(inner_blocks.remove(0))
                    } else {
                        None
                    }
                }
            }
        }

        AdocBlock::Preamble(preamble) => {
            // Preamble contains blocks before first section
            let mut inner_blocks = Vec::new();
            for child in preamble.nested_blocks() {
                if let Some(converted) = convert_block(child) {
                    inner_blocks.push(converted);
                }
            }
            if !inner_blocks.is_empty() {
                Some(inner_blocks.remove(0))
            } else {
                None
            }
        }

        AdocBlock::Break(_) => {
            Some(Block::ThematicBreak { span: None })
        }

        AdocBlock::Media(media) => {
            // Media blocks for images, video, audio
            let target = media.target()
                .map(|t| t.data().to_string())
                .unwrap_or_default();
            let alt = media.attrlist()
                .and_then(|a| a.named_or_positional_attribute("alt", 1))
                .map(|attr| attr.value().to_string())
                .unwrap_or_default();

            Some(Block::Paragraph {
                content: vec![Inline::Image {
                    url: target,
                    alt,
                    title: None,
                    width: None,
                    height: None,
                }],
                span: None,
            })
        }

        AdocBlock::DocumentAttribute(_) => {
            // Skip document attributes in block output
            None
        }

        // Non-exhaustive enum, handle any future variants
        _ => None,
    }
}

/// Parse inline content from text
fn parse_inline_content(text: &str) -> Vec<Inline> {
    // For now, just return as text
    // A full implementation would parse AsciiDoc inline markup
    vec![Inline::Text {
        content: text.to_string(),
    }]
}

impl Renderer for AsciidocHandler {
    fn format(&self) -> SourceFormat {
        SourceFormat::AsciiDoc
    }

    fn render(&self, doc: &Document, _config: &RenderConfig) -> Result<String> {
        let mut output = String::new();

        // Render title if present
        if let Some(ref title) = doc.meta.title {
            output.push_str("= ");
            output.push_str(title);
            output.push_str("\n\n");
        }

        for block in &doc.content {
            render_block(&mut output, block);
            output.push_str("\n\n");
        }

        // Trim trailing newlines
        while output.ends_with('\n') {
            output.pop();
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

        Block::Heading { level, content, id, .. } => {
            // AsciiDoc uses = for headings (= for level 1, == for level 2, etc.)
            output.push_str(&"=".repeat(*level as usize));
            output.push(' ');

            // Add ID anchor if present
            if let Some(ref id_str) = id {
                output.push_str(&format!("[[{}]] ", id_str));
            }

            for inline in content {
                render_inline(output, inline);
            }
        }

        Block::CodeBlock { language, content, .. } => {
            if let Some(ref lang) = language {
                output.push_str(&format!("[source,{}]\n", lang));
            }
            output.push_str("----\n");
            output.push_str(content);
            if !content.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("----");
        }

        Block::BlockQuote { content, attribution, .. } => {
            output.push_str("[quote");
            if let Some(ref attr) = attribution {
                output.push_str(", ");
                for inline in attr {
                    render_inline(output, inline);
                }
            }
            output.push_str("]\n____\n");
            for inner in content {
                render_block(output, inner);
                output.push('\n');
            }
            output.push_str("____");
        }

        Block::List { kind, items, start, .. } => {
            for (i, item) in items.iter().enumerate() {
                match kind {
                    ListKind::Bullet => output.push_str("* "),
                    ListKind::Ordered => {
                        let num = start.unwrap_or(1) + i as u32;
                        output.push_str(&format!("{}. ", num));
                    }
                    ListKind::Task => {
                        let checked = item.checked.unwrap_or(false);
                        output.push_str(if checked { "* [x] " } else { "* [ ] " });
                    }
                }
                for block in &item.content {
                    render_block(output, block);
                }
                output.push('\n');
            }
        }

        Block::ThematicBreak { .. } => {
            output.push_str("'''");
        }

        Block::Raw { content, .. } => {
            output.push_str("++++\n");
            output.push_str(content);
            if !content.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("++++");
        }

        Block::Container { classes, content, .. } => {
            // Render as sidebar if it has sidebar class
            if classes.contains(&"sidebar".to_string()) {
                output.push_str("****\n");
                for inner in content {
                    render_block(output, inner);
                    output.push('\n');
                }
                output.push_str("****");
            } else {
                // Generic container - render contents
                for inner in content {
                    render_block(output, inner);
                    output.push('\n');
                }
            }
        }

        Block::Table { header, body, caption, .. } => {
            output.push_str("|===\n");

            if let Some(h) = header {
                for cell in &h.cells {
                    output.push_str("| ");
                    for block in &cell.content {
                        render_block(output, block);
                    }
                    output.push(' ');
                }
                output.push_str("\n\n");
            }

            for row in body {
                for cell in &row.cells {
                    output.push_str("| ");
                    for block in &cell.content {
                        render_block(output, block);
                    }
                    output.push('\n');
                }
                output.push('\n');
            }

            output.push_str("|===");

            if let Some(ref cap) = caption {
                output.push_str("\n.");
                for inline in cap {
                    render_inline(output, inline);
                }
            }
        }

        Block::MathBlock { content, .. } => {
            output.push_str("[stem]\n++++\n");
            output.push_str(content);
            if !content.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("++++");
        }

        _ => {}
    }
}

fn render_inline(output: &mut String, inline: &Inline) {
    match inline {
        Inline::Text { content } => output.push_str(content),

        Inline::Emphasis { content } => {
            output.push('_');
            for i in content {
                render_inline(output, i);
            }
            output.push('_');
        }

        Inline::Strong { content } => {
            output.push('*');
            for i in content {
                render_inline(output, i);
            }
            output.push('*');
        }

        Inline::Strikethrough { content } => {
            output.push_str("[line-through]#");
            for i in content {
                render_inline(output, i);
            }
            output.push('#');
        }

        Inline::Code { content, .. } => {
            output.push('`');
            output.push_str(content);
            output.push('`');
        }

        Inline::Link { url, content, title, .. } => {
            output.push_str(&format!("link:{}[", url));
            for i in content {
                render_inline(output, i);
            }
            if let Some(ref t) = title {
                output.push_str(&format!(", title=\"{}\"", t));
            }
            output.push(']');
        }

        Inline::Image { url, alt, title, .. } => {
            output.push_str(&format!("image::{}[{}", url, alt));
            if let Some(ref t) = title {
                output.push_str(&format!(", title=\"{}\"", t));
            }
            output.push(']');
        }

        Inline::Math { content, .. } => {
            output.push_str("stem:[");
            output.push_str(content);
            output.push(']');
        }

        Inline::LineBreak => {
            output.push_str(" +\n");
        }

        Inline::SoftBreak => {
            output.push(' ');
        }

        _ => {}
    }
}

impl FormatHandler for AsciidocHandler {
    fn supports_feature(&self, feature: &str) -> bool {
        matches!(
            feature,
            "heading"
                | "bold"
                | "italic"
                | "strikethrough"
                | "code"
                | "code_block"
                | "link"
                | "image"
                | "list"
                | "table"
                | "math"
                | "admonition"
                | "footnote"
                | "cross_reference"
                | "include"
                | "macro"
        )
    }

    fn supported_features(&self) -> &[&str] {
        &[
            "heading",
            "bold",
            "italic",
            "strikethrough",
            "code",
            "code_block",
            "link",
            "image",
            "list",
            "table",
            "math",
            "admonition",
            "footnote",
            "cross_reference",
            "include",
            "macro",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let handler = AsciidocHandler::new();
        let result = handler.parse("Hello world", &ParseConfig::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_title() {
        let handler = AsciidocHandler::new();
        let input = "= Document Title\n\nThis is a paragraph.";
        let result = handler.parse(input, &ParseConfig::default());
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.meta.title, Some("Document Title".to_string()));
    }

    #[test]
    fn test_render_heading() {
        let handler = AsciidocHandler::new();
        let doc = Document {
            source_format: SourceFormat::AsciiDoc,
            meta: DocumentMeta::default(),
            content: vec![Block::Heading {
                level: 2,
                content: vec![Inline::Text { content: "Section Title".to_string() }],
                id: None,
                span: None,
            }],
            raw_source: None,
        };

        let output = handler.render(&doc, &RenderConfig::default()).unwrap();
        assert_eq!(output, "== Section Title");
    }

    #[test]
    fn test_render_code_block() {
        let handler = AsciidocHandler::new();
        let doc = Document {
            source_format: SourceFormat::AsciiDoc,
            meta: DocumentMeta::default(),
            content: vec![Block::CodeBlock {
                language: Some("rust".to_string()),
                content: "fn main() {}".to_string(),
                line_numbers: false,
                highlight_lines: Vec::new(),
                span: None,
            }],
            raw_source: None,
        };

        let output = handler.render(&doc, &RenderConfig::default()).unwrap();
        assert!(output.contains("[source,rust]"));
        assert!(output.contains("fn main()"));
    }
}
