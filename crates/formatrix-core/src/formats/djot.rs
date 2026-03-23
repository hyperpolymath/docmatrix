// SPDX-License-Identifier: PMPL-1.0-or-later
//! Djot format handler using jotdown

use crate::ast::{
    AdmonitionType, Block, Document, DocumentMeta, Inline, ListItem, ListKind, SourceFormat,
    TableCell, TableRow,
};
use crate::traits::{FormatHandler, ParseConfig, Parser, RenderConfig, Renderer, Result};
use jotdown::{Container, Event, Parser as JotdownParser};

/// Djot format handler using jotdown
pub struct DjotHandler;

impl DjotHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DjotHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for DjotHandler {
    fn format(&self) -> SourceFormat {
        SourceFormat::Djot
    }

    fn parse(&self, input: &str, config: &ParseConfig) -> Result<Document> {
        let parser = JotdownParser::new(input);
        let content = parse_events(parser);

        Ok(Document {
            source_format: SourceFormat::Djot,
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

/// Parse jotdown events into blocks
fn parse_events<'a>(parser: impl Iterator<Item = Event<'a>>) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut stack: Vec<(Container<'a>, Vec<Block>, Vec<Inline>)> = Vec::new();

    for event in parser {
        match event {
            Event::Start(container, _attrs) => {
                stack.push((container, Vec::new(), Vec::new()));
            }

            Event::End(container) => {
                if let Some((_, child_blocks, inlines)) = stack.pop() {
                    // Check if this is a Section - sections should pass through their children
                    if matches!(container, Container::Section { .. }) {
                        // Add all child blocks directly to parent or root
                        if let Some((_, parent_blocks, _)) = stack.last_mut() {
                            parent_blocks.extend(child_blocks);
                        } else {
                            blocks.extend(child_blocks);
                        }
                    } else {
                        let block = container_to_block(container, child_blocks, inlines);

                        if let Some(b) = block {
                            if let Some((_, parent_blocks, _)) = stack.last_mut() {
                                parent_blocks.push(b);
                            } else {
                                blocks.push(b);
                            }
                        }
                    }
                }
            }

            Event::Str(text) => {
                if let Some((_, _, inlines)) = stack.last_mut() {
                    inlines.push(Inline::Text {
                        content: text.to_string(),
                    });
                }
            }

            Event::Softbreak => {
                if let Some((_, _, inlines)) = stack.last_mut() {
                    inlines.push(Inline::SoftBreak);
                }
            }

            Event::Hardbreak => {
                if let Some((_, _, inlines)) = stack.last_mut() {
                    inlines.push(Inline::LineBreak);
                }
            }

            Event::NonBreakingSpace => {
                if let Some((_, _, inlines)) = stack.last_mut() {
                    inlines.push(Inline::Text {
                        content: "\u{00A0}".to_string(),
                    });
                }
            }

            Event::Escape => {
                // Escape sequences are handled by the parser
            }

            Event::Blankline => {
                // Blank lines between blocks
            }

            Event::ThematicBreak(_) => {
                if let Some((_, parent_blocks, _)) = stack.last_mut() {
                    parent_blocks.push(Block::ThematicBreak { span: None });
                } else {
                    blocks.push(Block::ThematicBreak { span: None });
                }
            }

            _ => {}
        }
    }

    blocks
}

/// Convert a jotdown container to a block
fn container_to_block(
    container: Container,
    child_blocks: Vec<Block>,
    inlines: Vec<Inline>,
) -> Option<Block> {
    match container {
        Container::Paragraph => Some(Block::Paragraph {
            content: inlines,
            span: None,
        }),

        Container::Heading { level, .. } => Some(Block::Heading {
            level: level as u8,
            content: inlines,
            id: None,
            span: None,
        }),

        Container::CodeBlock { language } => {
            let content = inlines
                .into_iter()
                .filter_map(|i| {
                    if let Inline::Text { content } = i {
                        Some(content)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("");

            Some(Block::CodeBlock {
                language: if language.is_empty() {
                    None
                } else {
                    Some(language.to_string())
                },
                content,
                line_numbers: false,
                highlight_lines: Vec::new(),
                span: None,
            })
        }

        Container::Blockquote => Some(Block::BlockQuote {
            content: if child_blocks.is_empty() {
                vec![Block::Paragraph {
                    content: inlines,
                    span: None,
                }]
            } else {
                child_blocks
            },
            attribution: None,
            admonition: None,
            span: None,
        }),

        Container::List { kind, .. } => {
            let list_kind = match kind {
                jotdown::ListKind::Unordered(_) => ListKind::Bullet,
                jotdown::ListKind::Ordered { .. } => ListKind::Ordered,
                jotdown::ListKind::Task(_) => ListKind::Task,
            };

            // Convert child blocks into list items
            let items: Vec<ListItem> = child_blocks
                .into_iter()
                .map(|block| ListItem {
                    content: vec![block],
                    checked: None,
                    marker: None,
                })
                .collect();

            Some(Block::List {
                kind: list_kind,
                items,
                start: None,
                span: None,
            })
        }

        Container::ListItem => {
            // Return the content as a paragraph to be wrapped by List
            if !child_blocks.is_empty() {
                Some(child_blocks.into_iter().next().unwrap())
            } else if !inlines.is_empty() {
                Some(Block::Paragraph {
                    content: inlines,
                    span: None,
                })
            } else {
                None
            }
        }

        Container::TaskListItem { checked: _ } => {
            let block = if !child_blocks.is_empty() {
                child_blocks.into_iter().next().unwrap()
            } else {
                Block::Paragraph {
                    content: inlines,
                    span: None,
                }
            };

            // We'll need to handle this specially in the List container
            Some(block)
        }

        Container::Table => {
            // Tables need special handling
            let mut header = None;
            let body = Vec::new();

            for block in child_blocks {
                if let Block::Raw { content, .. } = block {
                    // Parse table rows
                    if header.is_none() {
                        header = Some(TableRow {
                            cells: vec![TableCell {
                                content: vec![Block::Paragraph {
                                    content: vec![Inline::Text { content }],
                                    span: None,
                                }],
                                colspan: 1,
                                rowspan: 1,
                                alignment: None,
                            }],
                        });
                    }
                }
            }

            Some(Block::Table {
                caption: None,
                columns: Vec::new(),
                header,
                body,
                footer: None,
                span: None,
            })
        }

        Container::Div { class } => {
            // Check if it's an admonition
            let admonition = match class.as_ref() {
                "note" => Some(AdmonitionType::Note),
                "tip" => Some(AdmonitionType::Tip),
                "warning" => Some(AdmonitionType::Warning),
                "caution" => Some(AdmonitionType::Caution),
                "important" => Some(AdmonitionType::Important),
                _ => None,
            };

            if admonition.is_some() {
                Some(Block::BlockQuote {
                    content: child_blocks,
                    attribution: None,
                    admonition,
                    span: None,
                })
            } else {
                // Just return the child blocks as-is (simplified)
                child_blocks.into_iter().next()
            }
        }

        Container::Emphasis => {
            // This should produce inline, not block
            None
        }

        Container::Strong => None,

        Container::Link(_url, _link_type) => None,

        Container::Image(_url, _link_type) => None,

        Container::Footnote { label } => Some(Block::FootnoteDefinition {
            label: label.to_string(),
            content: child_blocks,
            span: None,
        }),

        Container::RawBlock { format: _ } => {
            let content = inlines
                .into_iter()
                .filter_map(|i| {
                    if let Inline::Text { content } = i {
                        Some(content)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("");

            Some(Block::Raw {
                format: SourceFormat::Djot,
                content,
                span: None,
            })
        }

        // Section is handled specially in the event loop
        Container::Section { .. } => None,

        _ => None,
    }
}

impl Renderer for DjotHandler {
    fn format(&self) -> SourceFormat {
        SourceFormat::Djot
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
    let prefix = " ".repeat(indent);

    match block {
        Block::Paragraph { content, .. } => {
            output.push_str(&prefix);
            for inline in content {
                render_inline(output, inline);
            }
        }

        Block::Heading {
            level, content, id, ..
        } => {
            output.push_str(&prefix);
            output.push_str(&"#".repeat(*level as usize));
            output.push(' ');
            for inline in content {
                render_inline(output, inline);
            }
            if let Some(id) = id {
                output.push_str(&format!(" {{#{}}}", id));
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

        Block::BlockQuote {
            content,
            admonition,
            ..
        } => {
            if let Some(admon) = admonition {
                output.push_str(&prefix);
                output.push_str("::: ");
                output.push_str(match admon {
                    AdmonitionType::Note => "note",
                    AdmonitionType::Tip => "tip",
                    AdmonitionType::Warning => "warning",
                    AdmonitionType::Caution => "caution",
                    AdmonitionType::Important => "important",
                    AdmonitionType::Danger => "danger",
                    AdmonitionType::Custom => "note",
                });
                output.push('\n');
                for block in content {
                    render_block(output, block, indent);
                    output.push('\n');
                }
                output.push_str(&prefix);
                output.push_str(":::");
            } else {
                for block in content {
                    output.push_str(&prefix);
                    output.push_str("> ");
                    render_block(output, block, 0);
                    output.push('\n');
                }
            }
        }

        Block::List {
            kind, items, start, ..
        } => {
            for (i, item) in items.iter().enumerate() {
                output.push_str(&prefix);
                match kind {
                    ListKind::Bullet => output.push_str("- "),
                    ListKind::Ordered => {
                        let num = start.unwrap_or(1) + i as u32;
                        output.push_str(&format!("{}. ", num));
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
            output.push_str("***");
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

        Block::FootnoteDefinition { label, content, .. } => {
            output.push_str(&prefix);
            output.push_str(&format!("[^{}]: ", label));
            for block in content {
                render_block(output, block, indent + 2);
            }
        }

        Block::Raw { content, .. } => {
            output.push_str(&prefix);
            output.push_str("```\n");
            output.push_str(content);
            output.push_str("\n```");
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
            output.push_str("*");
            for i in content {
                render_inline(output, i);
            }
            output.push_str("*");
        }

        Inline::Strikethrough { content } => {
            output.push_str("{-");
            for i in content {
                render_inline(output, i);
            }
            output.push_str("-}");
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

        Inline::Image {
            url, alt, title, ..
        } => {
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
            output.push_str("\\\n");
        }

        Inline::SoftBreak => {
            output.push('\n');
        }

        Inline::RawInline { content, .. } => {
            output.push_str("`");
            output.push_str(content);
            output.push_str("`");
        }

        _ => {}
    }
}

impl FormatHandler for DjotHandler {
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
                | "admonition"
                | "attributes"
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
            "admonition",
            "attributes",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heading() {
        let handler = DjotHandler::new();
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
        let handler = DjotHandler::new();
        let doc = handler
            .parse("Hello world", &ParseConfig::default())
            .unwrap();

        assert_eq!(doc.content.len(), 1);
        assert!(matches!(&doc.content[0], Block::Paragraph { .. }));
    }

    #[test]
    fn test_roundtrip_simple() {
        let handler = DjotHandler::new();
        let input = "# Heading\n\nParagraph text.";
        let doc = handler.parse(input, &ParseConfig::default()).unwrap();
        let output = handler.render(&doc, &RenderConfig::default()).unwrap();

        // Verify structure is preserved
        assert!(output.contains("# Heading"));
        assert!(output.contains("Paragraph text"));
    }
}
