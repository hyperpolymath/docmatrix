// SPDX-License-Identifier: PMPL-1.0-or-later
//! Markdown format handler using comrak

use crate::ast::{
    AdmonitionType, Block, Document, DocumentMeta, Inline, LinkType,
    ListItem, ListKind, SourceFormat, TableCell, TableRow,
};
use crate::traits::{FormatHandler, ParseConfig, Parser, RenderConfig, Renderer, Result};
use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, Options};

/// Markdown format handler using comrak (GFM-compatible)
pub struct MarkdownHandler;

impl MarkdownHandler {
    pub fn new() -> Self {
        Self
    }

    fn comrak_options() -> Options<'static> {
        let mut options = Options::default();
        options.extension.strikethrough = true;
        options.extension.table = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;
        options.extension.footnotes = true;
        options.extension.description_lists = true;
        options.parse.smart = true;
        options
    }
}

impl Default for MarkdownHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for MarkdownHandler {
    fn format(&self) -> SourceFormat {
        SourceFormat::Markdown
    }

    fn parse(&self, input: &str, config: &ParseConfig) -> Result<Document> {
        let arena = Arena::new();
        let options = Self::comrak_options();
        let root = parse_document(&arena, input, &options);

        let content = parse_children(root);

        Ok(Document {
            source_format: SourceFormat::Markdown,
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

fn parse_children<'a>(node: &'a AstNode<'a>) -> Vec<Block> {
    node.children()
        .filter_map(|child| parse_node(child))
        .collect()
}

fn parse_node<'a>(node: &'a AstNode<'a>) -> Option<Block> {
    let data = node.data.borrow();

    match &data.value {
        NodeValue::Document => None,

        NodeValue::Paragraph => Some(Block::Paragraph {
            content: parse_inlines(node),
            span: None,
        }),

        NodeValue::Heading(heading) => Some(Block::Heading {
            level: heading.level,
            content: parse_inlines(node),
            id: None,
            span: None,
        }),

        NodeValue::CodeBlock(code) => Some(Block::CodeBlock {
            language: if code.info.is_empty() {
                None
            } else {
                Some(code.info.clone())
            },
            content: code.literal.clone(),
            line_numbers: false,
            highlight_lines: Vec::new(),
            span: None,
        }),

        NodeValue::BlockQuote => Some(Block::BlockQuote {
            content: parse_children(node),
            attribution: None,
            admonition: detect_admonition(node),
            span: None,
        }),

        NodeValue::List(list) => {
            let kind = if list.list_type == comrak::nodes::ListType::Ordered {
                ListKind::Ordered
            } else {
                // Check if it's a task list
                let is_task = node.children().any(|child| {
                    matches!(
                        child.data.borrow().value,
                        NodeValue::TaskItem { .. }
                    )
                });
                if is_task {
                    ListKind::Task
                } else {
                    ListKind::Bullet
                }
            };

            let items: Vec<ListItem> = node
                .children()
                .map(|child| {
                    let checked = match child.data.borrow().value {
                        NodeValue::TaskItem(symbol) => Some(symbol.is_some()),
                        _ => None,
                    };
                    ListItem {
                        content: parse_children(child),
                        checked,
                        marker: None,
                    }
                })
                .collect();

            Some(Block::List {
                kind,
                items,
                start: if list.start > 1 {
                    Some(list.start as u32)
                } else {
                    None
                },
                span: None,
            })
        }

        NodeValue::Item(_) => None, // Handled by List

        NodeValue::TaskItem(_) => None, // Handled by List

        NodeValue::ThematicBreak => Some(Block::ThematicBreak { span: None }),

        NodeValue::Table(_) => {
            let mut header = None;
            let mut body = Vec::new();
            let columns = Vec::new(); // Would need to extract from table alignments

            for child in node.children() {
                match child.data.borrow().value {
                    NodeValue::TableRow(is_header) => {
                        let cells: Vec<TableCell> = child
                            .children()
                            .map(|cell| TableCell {
                                content: vec![Block::Paragraph {
                                    content: parse_inlines(cell),
                                    span: None,
                                }],
                                colspan: 1,
                                rowspan: 1,
                                alignment: None,
                            })
                            .collect();

                        let row = TableRow { cells };
                        if is_header {
                            header = Some(row);
                        } else {
                            body.push(row);
                        }
                    }
                    _ => {}
                }
            }

            Some(Block::Table {
                caption: None,
                columns,
                header,
                body,
                footer: None,
                span: None,
            })
        }

        NodeValue::FootnoteDefinition(def) => Some(Block::FootnoteDefinition {
            label: def.name.clone(),
            content: parse_children(node),
            span: None,
        }),

        NodeValue::HtmlBlock(html) => Some(Block::Raw {
            format: SourceFormat::Markdown,
            content: html.literal.clone(),
            span: None,
        }),

        _ => None,
    }
}

fn parse_inlines<'a>(node: &'a AstNode<'a>) -> Vec<Inline> {
    node.children()
        .filter_map(|child| parse_inline(child))
        .collect()
}

fn parse_inline<'a>(node: &'a AstNode<'a>) -> Option<Inline> {
    let data = node.data.borrow();

    match &data.value {
        NodeValue::Text(text) => Some(Inline::Text {
            content: text.clone(),
        }),

        NodeValue::SoftBreak => Some(Inline::SoftBreak),

        NodeValue::LineBreak => Some(Inline::LineBreak),

        NodeValue::Code(code) => Some(Inline::Code {
            content: code.literal.clone(),
            language: None,
        }),

        NodeValue::Emph => Some(Inline::Emphasis {
            content: parse_inlines(node),
        }),

        NodeValue::Strong => Some(Inline::Strong {
            content: parse_inlines(node),
        }),

        NodeValue::Strikethrough => Some(Inline::Strikethrough {
            content: parse_inlines(node),
        }),

        NodeValue::Link(link) => Some(Inline::Link {
            url: link.url.clone(),
            title: if link.title.is_empty() {
                None
            } else {
                Some(link.title.clone())
            },
            content: parse_inlines(node),
            link_type: LinkType::Inline,
        }),

        NodeValue::Image(image) => Some(Inline::Image {
            url: image.url.clone(),
            alt: node
                .children()
                .filter_map(|c| {
                    if let NodeValue::Text(t) = &c.data.borrow().value {
                        Some(t.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(""),
            title: if image.title.is_empty() {
                None
            } else {
                Some(image.title.clone())
            },
            width: None,
            height: None,
        }),

        NodeValue::FootnoteReference(fr) => Some(Inline::FootnoteRef {
            label: fr.name.clone(),
        }),

        NodeValue::HtmlInline(html) => Some(Inline::RawInline {
            format: SourceFormat::Markdown,
            content: html.clone(),
        }),

        _ => None,
    }
}

fn detect_admonition<'a>(_node: &'a AstNode<'a>) -> Option<AdmonitionType> {
    // GFM doesn't have native admonitions, but we could detect patterns like:
    // > [!NOTE]
    // > [!WARNING]
    // This is a simplified implementation
    None
}

impl Renderer for MarkdownHandler {
    fn format(&self) -> SourceFormat {
        SourceFormat::Markdown
    }

    fn render(&self, doc: &Document, _config: &RenderConfig) -> Result<String> {
        let mut output = String::new();

        for (i, block) in doc.content.iter().enumerate() {
            if i > 0 {
                output.push_str("\n\n");
            }
            render_block(&mut output, block, 0);
        }

        Ok(output)
    }
}

fn render_block(output: &mut String, block: &Block, indent: usize) {
    let prefix = "  ".repeat(indent);

    match block {
        Block::Paragraph { content, .. } => {
            output.push_str(&prefix);
            for inline in content {
                render_inline(output, inline);
            }
        }

        Block::Heading { level, content, .. } => {
            output.push_str(&prefix);
            output.push_str(&"#".repeat(*level as usize));
            output.push(' ');
            for inline in content {
                render_inline(output, inline);
            }
        }

        Block::CodeBlock {
            language, content, ..
        } => {
            output.push_str(&prefix);
            output.push_str("```");
            if let Some(lang) = language {
                output.push_str(lang);
            }
            output.push('\n');
            for line in content.lines() {
                output.push_str(&prefix);
                output.push_str(line);
                output.push('\n');
            }
            output.push_str(&prefix);
            output.push_str("```");
        }

        Block::BlockQuote { content, .. } => {
            for block in content {
                output.push_str(&prefix);
                output.push_str("> ");
                render_block(output, block, 0);
                output.push('\n');
            }
        }

        Block::List { kind, items, .. } => {
            for (i, item) in items.iter().enumerate() {
                output.push_str(&prefix);
                match kind {
                    ListKind::Bullet => output.push_str("- "),
                    ListKind::Ordered => {
                        output.push_str(&format!("{}. ", i + 1));
                    }
                    ListKind::Task => {
                        let checked = item.checked.unwrap_or(false);
                        output.push_str(if checked { "- [x] " } else { "- [ ] " });
                    }
                }
                for block in &item.content {
                    render_block(output, block, 0);
                }
                output.push('\n');
            }
        }

        Block::ThematicBreak { .. } => {
            output.push_str(&prefix);
            output.push_str("---");
        }

        Block::Table { header, body, .. } => {
            if let Some(h) = header {
                output.push_str(&prefix);
                output.push('|');
                for cell in &h.cells {
                    output.push(' ');
                    for block in &cell.content {
                        render_block(output, block, 0);
                    }
                    output.push_str(" |");
                }
                output.push('\n');

                // Separator
                output.push_str(&prefix);
                output.push('|');
                for _ in &h.cells {
                    output.push_str(" --- |");
                }
                output.push('\n');
            }

            for row in body {
                output.push_str(&prefix);
                output.push('|');
                for cell in &row.cells {
                    output.push(' ');
                    for block in &cell.content {
                        render_block(output, block, 0);
                    }
                    output.push_str(" |");
                }
                output.push('\n');
            }
        }

        Block::Raw { content, .. } => {
            output.push_str(content);
        }

        Block::FootnoteDefinition { label, content, .. } => {
            output.push_str(&prefix);
            output.push_str(&format!("[^{}]: ", label));
            for block in content {
                render_block(output, block, indent + 1);
            }
        }

        _ => {}
    }
}

fn render_inline(output: &mut String, inline: &Inline) {
    match inline {
        Inline::Text { content } => output.push_str(content),

        Inline::Emphasis { content } => {
            output.push('*');
            for i in content {
                render_inline(output, i);
            }
            output.push('*');
        }

        Inline::Strong { content } => {
            output.push_str("**");
            for i in content {
                render_inline(output, i);
            }
            output.push_str("**");
        }

        Inline::Strikethrough { content } => {
            output.push_str("~~");
            for i in content {
                render_inline(output, i);
            }
            output.push_str("~~");
        }

        Inline::Code { content, .. } => {
            output.push('`');
            output.push_str(content);
            output.push('`');
        }

        Inline::Link {
            url,
            title,
            content,
            ..
        } => {
            output.push('[');
            for i in content {
                render_inline(output, i);
            }
            output.push_str("](");
            output.push_str(url);
            if let Some(t) = title {
                output.push_str(&format!(" \"{}\"", t));
            }
            output.push(')');
        }

        Inline::Image { url, alt, title, .. } => {
            output.push_str("![");
            output.push_str(alt);
            output.push_str("](");
            output.push_str(url);
            if let Some(t) = title {
                output.push_str(&format!(" \"{}\"", t));
            }
            output.push(')');
        }

        Inline::FootnoteRef { label } => {
            output.push_str(&format!("[^{}]", label));
        }

        Inline::LineBreak => {
            output.push_str("  \n");
        }

        Inline::SoftBreak => {
            output.push('\n');
        }

        Inline::RawInline { content, .. } => {
            output.push_str(content);
        }

        _ => {}
    }
}

impl FormatHandler for MarkdownHandler {
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
                | "task_list"
                | "table"
                | "blockquote"
                | "footnote"
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
            "task_list",
            "table",
            "blockquote",
            "footnote",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heading() {
        let handler = MarkdownHandler::new();
        let doc = handler
            .parse("# Hello World", &ParseConfig::default())
            .unwrap();

        assert_eq!(doc.content.len(), 1);
        if let Block::Heading { level, .. } = &doc.content[0] {
            assert_eq!(*level, 1);
        } else {
            panic!("Expected heading");
        }
    }

    #[test]
    fn test_parse_paragraph() {
        let handler = MarkdownHandler::new();
        let doc = handler
            .parse("Hello **world**", &ParseConfig::default())
            .unwrap();

        assert_eq!(doc.content.len(), 1);
    }
}
