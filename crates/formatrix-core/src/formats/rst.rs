// SPDX-License-Identifier: PMPL-1.0-or-later
//! reStructuredText format handler using rst_parser
//! FD-S02: SHOULD requirement

use crate::ast::{
    AdmonitionType, Block, Document, DocumentMeta, Inline,
    LinkType, ListItem, ListKind, MathNotation, SourceFormat,
};
use crate::traits::{ConversionError, FormatHandler, ParseConfig, Parser, RenderConfig, Renderer, Result};
use rst_parser::parse;
use document_tree::{
    Document as RstDoc, HasChildren,
    element_categories::{BodyElement, StructuralSubElement, SubStructure, TextOrInlineElement},
};

/// reStructuredText format handler
pub struct RstHandler;

impl RstHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RstHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for RstHandler {
    fn format(&self) -> SourceFormat {
        SourceFormat::ReStructuredText
    }

    fn parse(&self, input: &str, config: &ParseConfig) -> Result<Document> {
        let rst_doc = parse(input).map_err(|e| {
            ConversionError::ParseError {
                line: 0,
                column: 0,
                message: format!("RST parse error: {:?}", e),
            }
        })?;

        let content = convert_rst_document(&rst_doc);

        Ok(Document {
            source_format: SourceFormat::ReStructuredText,
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

/// Convert RST document to our AST
fn convert_rst_document(doc: &RstDoc) -> Vec<Block> {
    let mut blocks = Vec::new();

    for child in doc.children() {
        convert_structural_element(&mut blocks, child);
    }

    blocks
}

/// Convert a structural sub-element to blocks
fn convert_structural_element(blocks: &mut Vec<Block>, element: &StructuralSubElement) {
    match element {
        StructuralSubElement::Title(title) => {
            let inlines = convert_text_elements(title.children());
            blocks.push(Block::Heading {
                level: 1,
                content: inlines,
                id: None,
                span: None,
            });
        }
        StructuralSubElement::Subtitle(subtitle) => {
            let inlines = convert_text_elements(subtitle.children());
            blocks.push(Block::Heading {
                level: 2,
                content: inlines,
                id: None,
                span: None,
            });
        }
        StructuralSubElement::SubStructure(sub) => {
            convert_substructure(blocks, sub);
        }
        _ => {}
    }
}

/// Convert a SubStructure element
fn convert_substructure(blocks: &mut Vec<Block>, sub: &SubStructure) {
    match sub {
        SubStructure::BodyElement(be) => {
            if let Some(block) = convert_body_element(be) {
                blocks.push(block);
            }
        }
        SubStructure::Section(section) => {
            for child in section.children() {
                convert_structural_element(blocks, child);
            }
        }
        SubStructure::Transition(_) => {
            blocks.push(Block::ThematicBreak { span: None });
        }
        _ => {}
    }
}

/// Convert a body element to a block
fn convert_body_element(element: &BodyElement) -> Option<Block> {
    match element {
        BodyElement::Paragraph(p) => {
            let inlines = convert_text_elements(p.children());
            Some(Block::Paragraph {
                content: inlines,
                span: None,
            })
        }

        BodyElement::LiteralBlock(lb) => {
            let content = extract_text_content(lb.children());
            Some(Block::CodeBlock {
                language: None,
                content,
                line_numbers: false,
                highlight_lines: Vec::new(),
                span: None,
            })
        }

        BodyElement::BlockQuote(bq) => {
            let mut inner_blocks = Vec::new();
            for child in bq.children() {
                match child {
                    document_tree::element_categories::SubBlockQuote::BodyElement(be) => {
                        if let Some(block) = convert_body_element(be) {
                            inner_blocks.push(block);
                        }
                    }
                    _ => {}
                }
            }
            Some(Block::BlockQuote {
                content: inner_blocks,
                attribution: None,
                admonition: None,
                span: None,
            })
        }

        BodyElement::BulletList(bl) => {
            let items: Vec<ListItem> = bl.children().iter().filter_map(|item| {
                let item_blocks: Vec<Block> = item.children().iter().filter_map(|child| {
                    convert_body_element(child)
                }).collect();

                Some(ListItem {
                    content: item_blocks,
                    checked: None,
                    marker: None,
                })
            }).collect();

            Some(Block::List {
                kind: ListKind::Bullet,
                items,
                start: None,
                span: None,
            })
        }

        BodyElement::EnumeratedList(el) => {
            let items: Vec<ListItem> = el.children().iter().filter_map(|item| {
                let item_blocks: Vec<Block> = item.children().iter().filter_map(|child| {
                    convert_body_element(child)
                }).collect();

                Some(ListItem {
                    content: item_blocks,
                    checked: None,
                    marker: None,
                })
            }).collect();

            Some(Block::List {
                kind: ListKind::Ordered,
                items,
                start: Some(1),
                span: None,
            })
        }

        BodyElement::Note(n) => {
            let inner_blocks: Vec<Block> = n.children().iter().filter_map(convert_body_element).collect();
            Some(Block::BlockQuote {
                content: inner_blocks,
                attribution: None,
                admonition: Some(AdmonitionType::Note),
                span: None,
            })
        }

        BodyElement::Warning(w) => {
            let inner_blocks: Vec<Block> = w.children().iter().filter_map(convert_body_element).collect();
            Some(Block::BlockQuote {
                content: inner_blocks,
                attribution: None,
                admonition: Some(AdmonitionType::Warning),
                span: None,
            })
        }

        BodyElement::Tip(t) => {
            let inner_blocks: Vec<Block> = t.children().iter().filter_map(convert_body_element).collect();
            Some(Block::BlockQuote {
                content: inner_blocks,
                attribution: None,
                admonition: Some(AdmonitionType::Tip),
                span: None,
            })
        }

        BodyElement::Important(i) => {
            let inner_blocks: Vec<Block> = i.children().iter().filter_map(convert_body_element).collect();
            Some(Block::BlockQuote {
                content: inner_blocks,
                attribution: None,
                admonition: Some(AdmonitionType::Important),
                span: None,
            })
        }

        BodyElement::Caution(c) => {
            let inner_blocks: Vec<Block> = c.children().iter().filter_map(convert_body_element).collect();
            Some(Block::BlockQuote {
                content: inner_blocks,
                attribution: None,
                admonition: Some(AdmonitionType::Caution),
                span: None,
            })
        }

        BodyElement::Danger(d) => {
            let inner_blocks: Vec<Block> = d.children().iter().filter_map(convert_body_element).collect();
            Some(Block::BlockQuote {
                content: inner_blocks,
                attribution: None,
                admonition: Some(AdmonitionType::Danger),
                span: None,
            })
        }

        BodyElement::MathBlock(m) => {
            let content = m.children().iter().map(|s| s.as_str()).collect::<Vec<_>>().join("");
            Some(Block::MathBlock {
                content,
                notation: MathNotation::LaTeX,
                span: None,
            })
        }

        _ => None,
    }
}

/// Convert TextOrInlineElement list to our Inline types
fn convert_text_elements(elements: &[TextOrInlineElement]) -> Vec<Inline> {
    let mut inlines = Vec::new();

    for elem in elements {
        match elem {
            TextOrInlineElement::String(s) => {
                inlines.push(Inline::Text {
                    content: (**s).clone(),
                });
            }
            TextOrInlineElement::Emphasis(e) => {
                let inner = convert_text_elements(e.children());
                inlines.push(Inline::Emphasis { content: inner });
            }
            TextOrInlineElement::Strong(s) => {
                let inner = convert_text_elements(s.children());
                inlines.push(Inline::Strong { content: inner });
            }
            TextOrInlineElement::Literal(l) => {
                let content = l.children().iter().map(|s| s.as_str()).collect::<Vec<_>>().join("");
                inlines.push(Inline::Code {
                    content,
                    language: None,
                });
            }
            TextOrInlineElement::Reference(r) => {
                let content = convert_text_elements(r.children());
                // RST references - use the first name as URL for now
                inlines.push(Inline::Link {
                    url: String::new(), // Will be resolved by transforms
                    title: None,
                    content,
                    link_type: LinkType::Reference,
                });
            }
            TextOrInlineElement::Superscript(sup) => {
                let inner = convert_text_elements(sup.children());
                inlines.push(Inline::Superscript { content: inner });
            }
            TextOrInlineElement::Subscript(sub) => {
                let inner = convert_text_elements(sub.children());
                inlines.push(Inline::Subscript { content: inner });
            }
            TextOrInlineElement::Math(m) => {
                let content = m.children().iter().map(|s| s.as_str()).collect::<Vec<_>>().join("");
                inlines.push(Inline::Math {
                    content,
                    notation: MathNotation::LaTeX,
                });
            }
            _ => {}
        }
    }

    inlines
}

/// Extract plain text content from TextOrInlineElement list
fn extract_text_content(elements: &[TextOrInlineElement]) -> String {
    elements.iter().map(|elem| {
        match elem {
            TextOrInlineElement::String(s) => (**s).clone(),
            _ => String::new(),
        }
    }).collect::<Vec<_>>().join("")
}

impl Renderer for RstHandler {
    fn format(&self) -> SourceFormat {
        SourceFormat::ReStructuredText
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

fn render_block(output: &mut String, block: &Block, _depth: usize) {
    match block {
        Block::Paragraph { content, .. } => {
            for inline in content {
                render_inline(output, inline);
            }
        }

        Block::Heading { level, content, .. } => {
            for inline in content {
                render_inline(output, inline);
            }
            output.push('\n');
            let underline = match level {
                1 => '=',
                2 => '-',
                3 => '~',
                4 => '^',
                _ => '\'',
            };
            let len = content.iter().map(|i| inline_text_len(i)).sum::<usize>();
            output.push_str(&underline.to_string().repeat(len.max(1)));
        }

        Block::CodeBlock { content, language, .. } => {
            if let Some(lang) = language {
                output.push_str(&format!(".. code-block:: {}\n\n", lang));
            } else {
                output.push_str("::\n\n");
            }
            for line in content.lines() {
                output.push_str("   ");
                output.push_str(line);
                output.push('\n');
            }
        }

        Block::BlockQuote { content, admonition, .. } => {
            if let Some(admon) = admonition {
                let directive = match admon {
                    AdmonitionType::Note => "note",
                    AdmonitionType::Tip => "tip",
                    AdmonitionType::Warning => "warning",
                    AdmonitionType::Caution => "caution",
                    AdmonitionType::Important => "important",
                    AdmonitionType::Danger => "danger",
                    AdmonitionType::Custom => "admonition",
                };
                output.push_str(&format!(".. {}::\n\n", directive));
                for block in content {
                    output.push_str("   ");
                    render_block(output, block, 1);
                    output.push('\n');
                }
            } else {
                for block in content {
                    output.push_str("   ");
                    render_block(output, block, 1);
                    output.push('\n');
                }
            }
        }

        Block::List { kind, items, .. } => {
            for (i, item) in items.iter().enumerate() {
                match kind {
                    ListKind::Bullet => output.push_str("* "),
                    ListKind::Ordered => output.push_str(&format!("{}. ", i + 1)),
                    ListKind::Task => {
                        let checked = item.checked.unwrap_or(false);
                        output.push_str(if checked { "[x] " } else { "[ ] " });
                    }
                }
                for (j, block) in item.content.iter().enumerate() {
                    if j > 0 {
                        output.push_str("\n   ");
                    }
                    render_block(output, block, 0);
                }
                output.push('\n');
            }
        }

        Block::ThematicBreak { .. } => {
            output.push_str("----");
        }

        Block::MathBlock { content, .. } => {
            output.push_str(".. math::\n\n");
            for line in content.lines() {
                output.push_str("   ");
                output.push_str(line);
                output.push('\n');
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

        Inline::Code { content, .. } => {
            output.push_str("``");
            output.push_str(content);
            output.push_str("``");
        }

        Inline::Link { url, content, .. } => {
            output.push('`');
            for i in content {
                render_inline(output, i);
            }
            output.push_str(" <");
            output.push_str(url);
            output.push_str(">`_");
        }

        Inline::Image { url, alt, .. } => {
            output.push_str(&format!(".. image:: {}\n   :alt: {}", url, alt));
        }

        Inline::Math { content, .. } => {
            output.push_str(":math:`");
            output.push_str(content);
            output.push('`');
        }

        Inline::Superscript { content } => {
            output.push_str(":sup:`");
            for i in content {
                render_inline(output, i);
            }
            output.push('`');
        }

        Inline::Subscript { content } => {
            output.push_str(":sub:`");
            for i in content {
                render_inline(output, i);
            }
            output.push('`');
        }

        Inline::LineBreak => {
            output.push_str("\n");
        }

        Inline::SoftBreak => {
            output.push(' ');
        }

        _ => {}
    }
}

fn inline_text_len(inline: &Inline) -> usize {
    match inline {
        Inline::Text { content } => content.len(),
        Inline::Emphasis { content } | Inline::Strong { content } => {
            content.iter().map(inline_text_len).sum()
        }
        Inline::Code { content, .. } => content.len(),
        _ => 0,
    }
}

impl FormatHandler for RstHandler {
    fn supports_feature(&self, feature: &str) -> bool {
        matches!(
            feature,
            "heading"
                | "bold"
                | "italic"
                | "code"
                | "code_block"
                | "link"
                | "image"
                | "list"
                | "blockquote"
                | "admonition"
                | "directive"
                | "role"
                | "math"
        )
    }

    fn supported_features(&self) -> &[&str] {
        &[
            "heading",
            "bold",
            "italic",
            "code",
            "code_block",
            "link",
            "image",
            "list",
            "blockquote",
            "admonition",
            "directive",
            "role",
            "math",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_paragraph() {
        let handler = RstHandler::new();
        let result = handler.parse("Hello world", &ParseConfig::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_heading() {
        let handler = RstHandler::new();
        let doc = Document {
            source_format: SourceFormat::ReStructuredText,
            meta: DocumentMeta::default(),
            content: vec![Block::Heading {
                level: 1,
                content: vec![Inline::Text { content: "Title".to_string() }],
                id: None,
                span: None,
            }],
            raw_source: None,
        };

        let output = handler.render(&doc, &RenderConfig::default()).unwrap();
        assert!(output.contains("Title"));
        assert!(output.contains("====="));
    }
}
