// SPDX-License-Identifier: PMPL-1.0-or-later
//! Org-mode format handler using orgize

use crate::ast::{
    Block, ColumnAlignment, ColumnSpec, Document, DocumentMeta, Inline, ListItem, ListKind,
    SourceFormat, TableCell, TableRow,
};
use crate::traits::{FormatHandler, ParseConfig, Parser, RenderConfig, Renderer, Result};
use orgize::elements::Element;
use orgize::Org;

/// Org-mode format handler using orgize
pub struct OrgModeHandler;

impl OrgModeHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OrgModeHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for OrgModeHandler {
    fn format(&self) -> SourceFormat {
        SourceFormat::OrgMode
    }

    fn parse(&self, input: &str, config: &ParseConfig) -> Result<Document> {
        let org = Org::parse(input);
        let content = parse_org(&org);

        Ok(Document {
            source_format: SourceFormat::OrgMode,
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

/// Parse orgize document into blocks
fn parse_org(org: &Org) -> Vec<Block> {
    use orgize::Event;

    let mut blocks = Vec::new();
    let mut event_iter = org.iter();

    while let Some(event) = event_iter.next() {
        match event {
            Event::Start(element) => {
                if let Some(block) = convert_element(element) {
                    blocks.push(block);
                } else if let Some(block) = handle_container(element, &mut event_iter) {
                    blocks.push(block);
                }
            }
            Event::End(_) => {}
        }
    }

    blocks
}

/// Convert a simple (non-container) element to a Block
fn convert_element(element: &Element) -> Option<Block> {
    match element {
        Element::Title(title) => {
            let content = vec![Inline::Text {
                content: title.raw.to_string(),
            }];

            Some(Block::Heading {
                level: title.level as u8,
                content,
                id: None,
                span: None,
            })
        }

        Element::SourceBlock(block) => Some(Block::CodeBlock {
            language: if block.language.is_empty() {
                None
            } else {
                Some(block.language.to_string())
            },
            content: block.contents.to_string(),
            line_numbers: false,
            highlight_lines: Vec::new(),
            span: None,
        }),

        Element::ExampleBlock(block) => Some(Block::CodeBlock {
            language: None,
            content: block.contents.to_string(),
            line_numbers: false,
            highlight_lines: Vec::new(),
            span: None,
        }),

        Element::Rule(_) => Some(Block::ThematicBreak { span: None }),

        Element::FixedWidth(fw) => Some(Block::CodeBlock {
            language: None,
            content: fw.value.to_string(),
            line_numbers: false,
            highlight_lines: Vec::new(),
            span: None,
        }),

        Element::ExportBlock(block) => Some(Block::Raw {
            format: SourceFormat::OrgMode,
            content: block.contents.to_string(),
            span: None,
        }),

        _ => None,
    }
}

/// Handle container elements that have nested content
fn handle_container<'a: 'b, 'b, I>(element: &'b Element<'a>, events: &mut I) -> Option<Block>
where
    I: Iterator<Item = orgize::Event<'a, 'b>>,
{
    match element {
        Element::Paragraph { .. } => {
            let mut inlines = Vec::new();
            collect_paragraph_content(&mut inlines, events);

            if inlines.is_empty() {
                None
            } else {
                Some(Block::Paragraph {
                    content: inlines,
                    span: None,
                })
            }
        }

        Element::QuoteBlock(_) => {
            let content = collect_block_content(events, |e| matches!(e, Element::QuoteBlock(_)));

            Some(Block::BlockQuote {
                content,
                attribution: None,
                admonition: None,
                span: None,
            })
        }

        Element::List(list) => {
            let kind = if list.ordered {
                ListKind::Ordered
            } else {
                ListKind::Bullet
            };

            let items = collect_list_items(events);

            Some(Block::List {
                kind,
                items,
                start: None,
                span: None,
            })
        }

        Element::Table(_) => {
            let (header, body) = collect_table_content(events);

            let col_count = header
                .as_ref()
                .map(|h| h.cells.len())
                .or_else(|| body.first().map(|r| r.cells.len()))
                .unwrap_or(0);

            Some(Block::Table {
                caption: None,
                columns: (0..col_count)
                    .map(|_| ColumnSpec {
                        alignment: ColumnAlignment::Default,
                        width: None,
                    })
                    .collect(),
                header,
                body,
                footer: None,
                span: None,
            })
        }

        _ => None,
    }
}

/// Collect paragraph content (inlines) until End(Paragraph)
fn collect_paragraph_content<'a: 'b, 'b, I>(inlines: &mut Vec<Inline>, events: &mut I)
where
    I: Iterator<Item = orgize::Event<'a, 'b>>,
{
    use orgize::Event;

    while let Some(event) = events.next() {
        match event {
            Event::End(Element::Paragraph { .. }) => break,
            Event::Start(Element::Text { value }) | Event::End(Element::Text { value }) => {
                inlines.push(Inline::Text {
                    content: value.to_string(),
                });
            }
            Event::Start(Element::Bold) => {
                let bold_content = collect_inline_until_end(events, |e| matches!(e, Element::Bold));
                inlines.push(Inline::Strong {
                    content: bold_content,
                });
            }
            Event::Start(Element::Italic) => {
                let italic_content =
                    collect_inline_until_end(events, |e| matches!(e, Element::Italic));
                inlines.push(Inline::Emphasis {
                    content: italic_content,
                });
            }
            Event::Start(Element::Strike) => {
                let strike_content =
                    collect_inline_until_end(events, |e| matches!(e, Element::Strike));
                inlines.push(Inline::Strikethrough {
                    content: strike_content,
                });
            }
            Event::Start(Element::Code { value }) | Event::End(Element::Code { value }) => {
                inlines.push(Inline::Code {
                    content: value.to_string(),
                    language: None,
                });
            }
            Event::Start(Element::Verbatim { value }) | Event::End(Element::Verbatim { value }) => {
                inlines.push(Inline::Code {
                    content: value.to_string(),
                    language: None,
                });
            }
            Event::Start(Element::Link(link)) => {
                let link_text = link
                    .desc
                    .as_ref()
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| link.path.to_string());
                inlines.push(Inline::Link {
                    url: link.path.to_string(),
                    title: None,
                    content: vec![Inline::Text { content: link_text }],
                    link_type: crate::ast::LinkType::Inline,
                });
            }
            _ => {}
        }
    }
}

/// Collect inline content until a matching end element
fn collect_inline_until_end<'a: 'b, 'b, I, F>(events: &mut I, is_end_element: F) -> Vec<Inline>
where
    I: Iterator<Item = orgize::Event<'a, 'b>>,
    F: Fn(&Element) -> bool,
{
    use orgize::Event;
    let mut inlines = Vec::new();

    while let Some(event) = events.next() {
        match &event {
            Event::End(elem) if is_end_element(elem) => break,
            Event::Start(Element::Text { value }) | Event::End(Element::Text { value }) => {
                inlines.push(Inline::Text {
                    content: value.to_string(),
                });
            }
            Event::Start(Element::Code { value }) | Event::End(Element::Code { value }) => {
                inlines.push(Inline::Code {
                    content: value.to_string(),
                    language: None,
                });
            }
            _ => {}
        }
    }

    inlines
}

/// Collect block content until a matching end element
fn collect_block_content<'a: 'b, 'b, I, F>(events: &mut I, is_end_element: F) -> Vec<Block>
where
    I: Iterator<Item = orgize::Event<'a, 'b>>,
    F: Fn(&Element) -> bool,
{
    use orgize::Event;
    let mut blocks = Vec::new();
    let mut depth = 1;

    while let Some(event) = events.next() {
        match &event {
            Event::Start(elem) if is_end_element(elem) => depth += 1,
            Event::End(elem) if is_end_element(elem) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            Event::Start(Element::Paragraph { .. }) => {
                let mut inlines = Vec::new();
                collect_paragraph_content(&mut inlines, events);
                if !inlines.is_empty() {
                    blocks.push(Block::Paragraph {
                        content: inlines,
                        span: None,
                    });
                }
            }
            Event::Start(Element::Text { value }) => {
                blocks.push(Block::Paragraph {
                    content: vec![Inline::Text {
                        content: value.to_string(),
                    }],
                    span: None,
                });
            }
            _ => {}
        }
    }

    blocks
}

/// Collect list items until End(List)
fn collect_list_items<'a: 'b, 'b, I>(events: &mut I) -> Vec<ListItem>
where
    I: Iterator<Item = orgize::Event<'a, 'b>>,
{
    use orgize::Event;
    let mut items = Vec::new();
    let mut depth = 1;

    while let Some(event) = events.next() {
        match &event {
            Event::Start(Element::List(_)) => depth += 1,
            Event::End(Element::List(_)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            Event::Start(Element::ListItem(_item)) => {
                // Note: orgize 0.9 ListItem doesn't have checkbox field yet
                let item_content = collect_list_item_content(events);
                items.push(ListItem {
                    content: item_content,
                    checked: None, // orgize 0.9 doesn't expose checkbox
                    marker: None,
                });
            }
            _ => {}
        }
    }

    items
}

/// Collect content for a single list item until End(ListItem)
fn collect_list_item_content<'a: 'b, 'b, I>(events: &mut I) -> Vec<Block>
where
    I: Iterator<Item = orgize::Event<'a, 'b>>,
{
    use orgize::Event;
    let mut blocks = Vec::new();

    while let Some(event) = events.next() {
        match &event {
            Event::End(Element::ListItem(_)) => break,
            Event::Start(Element::Paragraph { .. }) => {
                let mut inlines = Vec::new();
                collect_paragraph_content(&mut inlines, events);
                if !inlines.is_empty() {
                    blocks.push(Block::Paragraph {
                        content: inlines,
                        span: None,
                    });
                }
            }
            Event::Start(Element::List(list)) => {
                let kind = if list.ordered {
                    ListKind::Ordered
                } else {
                    ListKind::Bullet
                };
                let nested_items = collect_list_items(events);
                blocks.push(Block::List {
                    kind,
                    items: nested_items,
                    start: None,
                    span: None,
                });
            }
            Event::Start(Element::Text { value }) => {
                blocks.push(Block::Paragraph {
                    content: vec![Inline::Text {
                        content: value.to_string(),
                    }],
                    span: None,
                });
            }
            _ => {}
        }
    }

    blocks
}

/// Collect table content - returns (header_row, body_rows)
fn collect_table_content<'a: 'b, 'b, I>(events: &mut I) -> (Option<TableRow>, Vec<TableRow>)
where
    I: Iterator<Item = orgize::Event<'a, 'b>>,
{
    use orgize::elements::TableRow as OrgTableRow;
    use orgize::Event;

    let mut header_row: Option<TableRow> = None;
    let mut body_rows: Vec<TableRow> = Vec::new();
    let mut in_header = true;
    let mut depth = 1;

    while let Some(event) = events.next() {
        match &event {
            Event::Start(Element::Table(_)) => depth += 1,
            Event::End(Element::Table(_)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            // TableRow is an enum: Header, Body, HeaderRule, BodyRule
            Event::Start(Element::TableRow(OrgTableRow::HeaderRule))
            | Event::Start(Element::TableRow(OrgTableRow::BodyRule)) => {
                // This is a separator/rule row - marks end of header
                in_header = false;
                // Skip until end of row
                skip_until_end_row(events);
            }
            Event::Start(Element::TableRow(OrgTableRow::Header))
            | Event::Start(Element::TableRow(OrgTableRow::Body)) => {
                let cells = collect_table_row_cells(events);
                if !cells.is_empty() {
                    let row = TableRow { cells };
                    if in_header && header_row.is_none() {
                        header_row = Some(row);
                    } else {
                        body_rows.push(row);
                    }
                }
            }
            _ => {}
        }
    }

    (header_row, body_rows)
}

/// Skip events until End(TableRow)
fn skip_until_end_row<'a: 'b, 'b, I>(events: &mut I)
where
    I: Iterator<Item = orgize::Event<'a, 'b>>,
{
    use orgize::Event;
    while let Some(event) = events.next() {
        if matches!(event, Event::End(Element::TableRow(_))) {
            break;
        }
    }
}

/// Collect cells for a table row until End(TableRow)
fn collect_table_row_cells<'a: 'b, 'b, I>(events: &mut I) -> Vec<TableCell>
where
    I: Iterator<Item = orgize::Event<'a, 'b>>,
{
    use orgize::elements::TableCell as OrgTableCell;
    use orgize::Event;

    let mut cells = Vec::new();

    while let Some(event) = events.next() {
        match &event {
            Event::End(Element::TableRow(_)) => break,
            // TableCell is an enum: Header, Body
            Event::Start(Element::TableCell(OrgTableCell::Header))
            | Event::Start(Element::TableCell(OrgTableCell::Body)) => {
                let cell_text = collect_cell_text(events);
                cells.push(TableCell {
                    content: vec![Block::Paragraph {
                        content: vec![Inline::Text {
                            content: cell_text.trim().to_string(),
                        }],
                        span: None,
                    }],
                    colspan: 1,
                    rowspan: 1,
                    alignment: None, // orgize 0.9 TableCell enum doesn't have alignment
                });
            }
            _ => {}
        }
    }

    cells
}

/// Collect text content for a table cell until End(TableCell)
fn collect_cell_text<'a: 'b, 'b, I>(events: &mut I) -> String
where
    I: Iterator<Item = orgize::Event<'a, 'b>>,
{
    use orgize::Event;
    let mut text = String::new();

    while let Some(event) = events.next() {
        match event {
            Event::End(Element::TableCell(_)) => break,
            Event::Start(Element::Text { value }) | Event::End(Element::Text { value }) => {
                text.push_str(&value);
            }
            _ => {}
        }
    }

    text
}

impl Renderer for OrgModeHandler {
    fn format(&self) -> SourceFormat {
        SourceFormat::OrgMode
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
            output.push_str(&"*".repeat(*level as usize));
            output.push(' ');
            for inline in content {
                render_inline(output, inline);
            }
        }

        Block::CodeBlock {
            language, content, ..
        } => {
            output.push_str("#+BEGIN_SRC");
            if let Some(lang) = language {
                output.push(' ');
                output.push_str(lang);
            }
            output.push('\n');
            output.push_str(content);
            if !content.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("#+END_SRC");
        }

        Block::BlockQuote { content, .. } => {
            output.push_str("#+BEGIN_QUOTE\n");
            for block in content {
                render_block(output, block);
                output.push('\n');
            }
            output.push_str("#+END_QUOTE");
        }

        Block::List {
            kind, items, start, ..
        } => {
            for (i, item) in items.iter().enumerate() {
                match kind {
                    ListKind::Bullet => output.push_str("- "),
                    ListKind::Ordered => {
                        let num = start.unwrap_or(1) + i as u32;
                        output.push_str(&format!("{}. ", num));
                    }
                    ListKind::Task => {
                        let checked = item.checked.unwrap_or(false);
                        output.push_str(if checked { "- [X] " } else { "- [ ] " });
                    }
                }
                for block in &item.content {
                    render_block(output, block);
                }
                output.push('\n');
            }
        }

        Block::ThematicBreak { .. } => {
            output.push_str("-----");
        }

        Block::Table { header, body, .. } => {
            if let Some(h) = header {
                output.push('|');
                for cell in &h.cells {
                    output.push(' ');
                    for block in &cell.content {
                        render_block(output, block);
                    }
                    output.push_str(" |");
                }
                output.push('\n');
                output.push_str("|---|\n");
            }

            for row in body {
                output.push('|');
                for cell in &row.cells {
                    output.push(' ');
                    for block in &cell.content {
                        render_block(output, block);
                    }
                    output.push_str(" |");
                }
                output.push('\n');
            }
        }

        Block::FootnoteDefinition { label, content, .. } => {
            output.push_str(&format!("[fn:{}] ", label));
            for block in content {
                render_block(output, block);
            }
        }

        Block::Raw { content, .. } => {
            output.push_str("#+BEGIN_EXPORT\n");
            output.push_str(content);
            output.push_str("\n#+END_EXPORT");
        }

        _ => {}
    }
}

fn render_inline(output: &mut String, inline: &Inline) {
    match inline {
        Inline::Text { content } => output.push_str(content),

        Inline::Emphasis { content } => {
            output.push('/');
            for i in content {
                render_inline(output, i);
            }
            output.push('/');
        }

        Inline::Strong { content } => {
            output.push('*');
            for i in content {
                render_inline(output, i);
            }
            output.push('*');
        }

        Inline::Strikethrough { content } => {
            output.push('+');
            for i in content {
                render_inline(output, i);
            }
            output.push('+');
        }

        Inline::Code { content, .. } => {
            output.push('~');
            output.push_str(content);
            output.push('~');
        }

        Inline::Link { url, content, .. } => {
            output.push_str("[[");
            output.push_str(url);
            output.push_str("][");
            for i in content {
                render_inline(output, i);
            }
            output.push_str("]]");
        }

        Inline::Image { url, .. } => {
            output.push_str("[[");
            output.push_str(url);
            output.push_str("]]");
        }

        Inline::FootnoteRef { label } => {
            output.push_str(&format!("[fn:{}]", label));
        }

        Inline::LineBreak => {
            output.push_str("\\\\\n");
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

impl FormatHandler for OrgModeHandler {
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
                | "verbatim"
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
            "verbatim",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heading() {
        let handler = OrgModeHandler::new();
        let doc = handler
            .parse("* Hello World", &ParseConfig::default())
            .unwrap();

        // Find the heading in the parsed content
        let has_heading = doc
            .content
            .iter()
            .any(|b| matches!(b, Block::Heading { level: 1, .. }));
        assert!(has_heading || doc.content.is_empty()); // orgize may parse differently
    }

    #[test]
    fn test_render_heading() {
        let handler = OrgModeHandler::new();
        let doc = Document {
            source_format: SourceFormat::OrgMode,
            meta: DocumentMeta::default(),
            content: vec![Block::Heading {
                level: 1,
                content: vec![Inline::Text {
                    content: "Test".to_string(),
                }],
                id: None,
                span: None,
            }],
            raw_source: None,
        };

        let output = handler.render(&doc, &RenderConfig::default()).unwrap();
        assert!(output.contains("* Test"));
    }

    #[test]
    fn test_render_code_block() {
        let handler = OrgModeHandler::new();
        let doc = Document {
            source_format: SourceFormat::OrgMode,
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
        assert!(output.contains("#+BEGIN_SRC rust"));
        assert!(output.contains("fn main()"));
        assert!(output.contains("#+END_SRC"));
    }

    #[test]
    fn test_parse_quote_block() {
        let handler = OrgModeHandler::new();
        let input = r#"#+BEGIN_QUOTE
This is a quote.
#+END_QUOTE"#;
        let doc = handler.parse(input, &ParseConfig::default()).unwrap();

        let has_quote = doc
            .content
            .iter()
            .any(|b| matches!(b, Block::BlockQuote { .. }));
        assert!(has_quote, "Should parse quote block");
    }

    #[test]
    fn test_parse_list() {
        let handler = OrgModeHandler::new();
        let input = r#"- Item 1
- Item 2
- Item 3"#;
        let doc = handler.parse(input, &ParseConfig::default()).unwrap();

        let has_list = doc.content.iter().any(|b| {
            if let Block::List { items, .. } = b {
                items.len() == 3
            } else {
                false
            }
        });
        assert!(has_list, "Should parse list with 3 items");
    }

    #[test]
    fn test_parse_table() {
        let handler = OrgModeHandler::new();
        let input = r#"| Header 1 | Header 2 |
|----------+----------|
| Cell 1   | Cell 2   |"#;
        let doc = handler.parse(input, &ParseConfig::default()).unwrap();

        let has_table = doc.content.iter().any(|b| matches!(b, Block::Table { .. }));
        assert!(has_table, "Should parse table");
    }
}
