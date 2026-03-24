// SPDX-License-Identifier: PMPL-1.0-or-later
//! Typst format handler using typst-syntax
//! FD-S03: SHOULD requirement

use crate::ast::{
    Block, Document, DocumentMeta, Inline, ListItem, ListKind, MathNotation, SourceFormat,
};
use crate::traits::{FormatHandler, ParseConfig, Parser, RenderConfig, Renderer, Result};
use typst_syntax::{parse, SyntaxKind, SyntaxNode};

/// Typst format handler
pub struct TypstHandler;

impl TypstHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TypstHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for TypstHandler {
    fn format(&self) -> SourceFormat {
        SourceFormat::Typst
    }

    fn parse(&self, input: &str, config: &ParseConfig) -> Result<Document> {
        let tree = parse(input);
        let content = parse_syntax_tree(&tree);

        Ok(Document {
            source_format: SourceFormat::Typst,
            meta: DocumentMeta::default(),
            content,
            raw_source: if config.preserve_raw_source {
                Some(input.to_string())
            } else {
                None
            },
        })
    }
}

/// Parse the Typst syntax tree into our AST
fn parse_syntax_tree(root: &SyntaxNode) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current_text = String::new();

    for child in root.children() {
        match child.kind() {
            SyntaxKind::Text => {
                current_text.push_str(child.text());
            }

            SyntaxKind::Space => {
                if !current_text.is_empty() {
                    current_text.push(' ');
                }
            }

            SyntaxKind::Parbreak => {
                if !current_text.trim().is_empty() {
                    blocks.push(Block::Paragraph {
                        content: vec![Inline::Text {
                            content: current_text.trim().to_string(),
                        }],
                        span: None,
                    });
                }
                current_text.clear();
            }

            SyntaxKind::Heading => {
                // Flush any pending text
                if !current_text.trim().is_empty() {
                    blocks.push(Block::Paragraph {
                        content: vec![Inline::Text {
                            content: current_text.trim().to_string(),
                        }],
                        span: None,
                    });
                    current_text.clear();
                }

                // Parse heading
                if let Some(heading) = parse_heading(&child) {
                    blocks.push(heading);
                }
            }

            SyntaxKind::ListItem => {
                // Handle list items
                if let Some(item) = parse_list_item(&child) {
                    // Check if we can append to existing list
                    if let Some(Block::List { items, .. }) = blocks.last_mut() {
                        items.push(item);
                    } else {
                        blocks.push(Block::List {
                            kind: ListKind::Bullet,
                            items: vec![item],
                            start: None,
                            span: None,
                        });
                    }
                }
            }

            SyntaxKind::EnumItem => {
                if let Some(item) = parse_list_item(&child) {
                    if let Some(Block::List {
                        kind: ListKind::Ordered,
                        items,
                        ..
                    }) = blocks.last_mut()
                    {
                        items.push(item);
                    } else {
                        blocks.push(Block::List {
                            kind: ListKind::Ordered,
                            items: vec![item],
                            start: Some(1),
                            span: None,
                        });
                    }
                }
            }

            SyntaxKind::Raw => {
                // Code block
                let content = extract_raw_content(&child);
                let language = extract_raw_language(&child);
                blocks.push(Block::CodeBlock {
                    language,
                    content,
                    line_numbers: false,
                    highlight_lines: Vec::new(),
                    span: None,
                });
            }

            SyntaxKind::Equation => {
                // Math equation - store as MathBlock
                let content = child.text().to_string();
                // Remove leading/trailing $ if present
                let content = content.trim_matches('$').trim().to_string();
                blocks.push(Block::MathBlock {
                    content,
                    notation: MathNotation::LaTeX,
                    span: None,
                });
            }

            SyntaxKind::Strong => {
                current_text.push_str(&format!("*{}*", extract_text(&child)));
            }

            SyntaxKind::Emph => {
                current_text.push_str(&format!("_{}_", extract_text(&child)));
            }

            SyntaxKind::Link => {
                let url = extract_text(&child);
                current_text.push_str(&url);
            }

            SyntaxKind::Markup => {
                // Recurse into markup content
                let inner_blocks = parse_syntax_tree(&child);
                blocks.extend(inner_blocks);
            }

            _ => {
                // For other nodes, try to extract text
                let text = child.text().to_string();
                if !text.is_empty() && !text.trim().is_empty() {
                    current_text.push_str(&text);
                }
            }
        }
    }

    // Flush remaining text
    if !current_text.trim().is_empty() {
        blocks.push(Block::Paragraph {
            content: vec![Inline::Text {
                content: current_text.trim().to_string(),
            }],
            span: None,
        });
    }

    blocks
}

/// Parse a heading node
fn parse_heading(node: &SyntaxNode) -> Option<Block> {
    let mut level = 1u8;
    let mut content = String::new();

    for child in node.children() {
        match child.kind() {
            SyntaxKind::HeadingMarker => {
                // Count = signs for level
                level = child.text().chars().filter(|c| *c == '=').count() as u8;
            }
            _ => {
                content.push_str(child.text());
            }
        }
    }

    Some(Block::Heading {
        level,
        content: vec![Inline::Text {
            content: content.trim().to_string(),
        }],
        id: None,
        span: None,
    })
}

/// Parse a list item
fn parse_list_item(node: &SyntaxNode) -> Option<ListItem> {
    let mut content_text = String::new();

    for child in node.children() {
        match child.kind() {
            SyntaxKind::ListMarker | SyntaxKind::EnumMarker => {
                // Skip markers
            }
            _ => {
                content_text.push_str(child.text());
            }
        }
    }

    Some(ListItem {
        content: vec![Block::Paragraph {
            content: vec![Inline::Text {
                content: content_text.trim().to_string(),
            }],
            span: None,
        }],
        checked: None,
        marker: None,
    })
}

/// Extract text from a node recursively
fn extract_text(node: &SyntaxNode) -> String {
    let mut text = String::new();
    for child in node.children() {
        text.push_str(child.text());
    }
    if text.is_empty() {
        text = node.text().to_string();
    }
    text.trim().to_string()
}

/// Extract content from a raw (code) block
fn extract_raw_content(node: &SyntaxNode) -> String {
    let text = node.text().to_string();
    // Remove the backticks
    text.trim_matches('`').to_string()
}

/// Extract language from a raw block (if specified)
fn extract_raw_language(node: &SyntaxNode) -> Option<String> {
    for child in node.children() {
        if child.kind() == SyntaxKind::Ident {
            return Some(child.text().to_string());
        }
    }
    None
}

impl Renderer for TypstHandler {
    fn format(&self) -> SourceFormat {
        SourceFormat::Typst
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

        Block::Heading { level, content, .. } => {
            output.push_str(&"=".repeat(*level as usize));
            output.push(' ');
            for inline in content {
                render_inline(output, inline);
            }
        }

        Block::CodeBlock {
            language, content, ..
        } => {
            output.push_str("```");
            if let Some(lang) = language {
                output.push_str(lang);
            }
            output.push('\n');
            output.push_str(content);
            if !content.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("```");
        }

        Block::BlockQuote { content, .. } => {
            output.push_str("#quote[\n");
            for block in content {
                output.push_str("  ");
                render_block(output, block);
                output.push('\n');
            }
            output.push(']');
        }

        Block::List { kind, items, .. } => {
            for (i, item) in items.iter().enumerate() {
                match kind {
                    ListKind::Bullet => output.push_str("- "),
                    ListKind::Ordered => output.push_str(&format!("{}. ", i + 1)),
                    ListKind::Task => {
                        // Typst doesn't have native task lists
                        let checked = item.checked.unwrap_or(false);
                        output.push_str(if checked { "- [x] " } else { "- [ ] " });
                    }
                }
                for block in &item.content {
                    render_block(output, block);
                }
                output.push('\n');
            }
        }

        Block::ThematicBreak { .. } => {
            output.push_str("#line(length: 100%)");
        }

        Block::MathBlock { content, .. } => {
            output.push_str("$ ");
            output.push_str(content);
            output.push_str(" $");
        }

        Block::Raw { content, .. } => {
            output.push_str("#raw[");
            output.push_str(content);
            output.push(']');
        }

        Block::Table {
            header,
            body,
            caption,
            ..
        } => {
            output.push_str("#table(\n");

            if let Some(h) = header {
                for cell in &h.cells {
                    output.push_str("  table.header[");
                    for block in &cell.content {
                        render_block(output, block);
                    }
                    output.push_str("],\n");
                }
            }

            for row in body {
                for cell in &row.cells {
                    output.push_str("  [");
                    for block in &cell.content {
                        render_block(output, block);
                    }
                    output.push_str("],\n");
                }
            }

            output.push(')');

            if let Some(cap) = caption {
                output.push_str("\n#figure.caption[");
                for inline in cap {
                    render_inline(output, inline);
                }
                output.push(']');
            }
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
            output.push_str("#strike[");
            for i in content {
                render_inline(output, i);
            }
            output.push(']');
        }

        Inline::Code { content, .. } => {
            output.push('`');
            output.push_str(content);
            output.push('`');
        }

        Inline::Link { url, content, .. } => {
            output.push_str("#link(\"");
            output.push_str(url);
            output.push_str("\")[");
            for i in content {
                render_inline(output, i);
            }
            output.push(']');
        }

        Inline::Image { url, alt, .. } => {
            output.push_str(&format!("#image(\"{}\", alt: \"{}\")", url, alt));
        }

        Inline::Math { content, notation } => {
            // Typst uses $ for inline math
            match notation {
                MathNotation::LaTeX | MathNotation::AsciiMath => {
                    output.push('$');
                    output.push_str(content);
                    output.push('$');
                }
                MathNotation::MathML => {
                    // MathML would need conversion, just output raw for now
                    output.push_str("#raw[");
                    output.push_str(content);
                    output.push(']');
                }
            }
        }

        Inline::LineBreak => {
            output.push_str("\\ \n");
        }

        Inline::SoftBreak => {
            output.push(' ');
        }

        _ => {}
    }
}

impl FormatHandler for TypstHandler {
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
                | "figure"
                | "bibliography"
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
            "figure",
            "bibliography",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let handler = TypstHandler::new();
        let result = handler.parse("Hello world", &ParseConfig::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_heading() {
        let handler = TypstHandler::new();
        let result = handler.parse("= Title", &ParseConfig::default());
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert!(!doc.content.is_empty());
    }

    #[test]
    fn test_render_heading() {
        let handler = TypstHandler::new();
        let doc = Document {
            source_format: SourceFormat::Typst,
            meta: DocumentMeta::default(),
            content: vec![Block::Heading {
                level: 1,
                content: vec![Inline::Text {
                    content: "Title".to_string(),
                }],
                id: None,
                span: None,
            }],
            raw_source: None,
        };

        let output = handler.render(&doc, &RenderConfig::default()).unwrap();
        assert_eq!(output, "= Title");
    }

    #[test]
    fn test_render_code_block() {
        let handler = TypstHandler::new();
        let doc = Document {
            source_format: SourceFormat::Typst,
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
        assert!(output.contains("```rust"));
        assert!(output.contains("fn main()"));
    }
}
