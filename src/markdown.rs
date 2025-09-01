use pulldown_cmark::{Parser, Event, Tag, Options, LinkType, CodeBlockKind, TaskListMarker};
use ratatui::style::{Style, Modifier};
use ratatui::text::Span;
use crate::colors::CatppuccinFrappe;

pub fn render_markdown(markdown_text: &str) -> Vec<Span> {
    // Enable all markdown extensions
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);
    
    let parser = Parser::new_ext(markdown_text, options);
    let mut spans = Vec::new();
    let mut current_style = Style::default().fg(CatppuccinFrappe::TEXT);
    let mut in_blockquote = false;
    let mut in_table = false;
    
    for event in parser {
        match event {
            Event::Text(text) => {
                // Apply blockquote styling if in a blockquote
                let style = if in_blockquote {
                    current_style.fg(CatppuccinFrappe::TEAL)
                } else {
                    current_style
                };
                spans.push(Span::styled(text.to_string(), style));
            },
            Event::Start(Tag::Heading(level)) => {
                // Different styling based on heading level
                let heading_color = match level {
                    1 => CatppuccinFrappe::BLUE,
                    2 => CatppuccinFrappe::LAVENDER,
                    _ => CatppuccinFrappe::MAUVE,
                };
                
                current_style = Style::default()
                    .fg(heading_color)
                    .add_modifier(Modifier::BOLD);
                
                // Add prefix based on heading level
                let prefix = match level {
                    1 => "# ",
                    2 => "## ",
                    3 => "### ",
                    4 => "#### ",
                    5 => "##### ",
                    _ => "###### ",
                };
                spans.push(Span::styled(prefix, current_style));
            },
            Event::End(Tag::Heading(_)) => {
                current_style = Style::default().fg(CatppuccinFrappe::TEXT);
                spans.push(Span::raw("\n"));
            },
            Event::Start(Tag::Paragraph) => {
                if !spans.is_empty() {
                    spans.push(Span::raw("\n"));
                }
            },
            Event::End(Tag::Paragraph) => {
                spans.push(Span::raw("\n"));
            },
            Event::Start(Tag::BlockQuote) => {
                in_blockquote = true;
                spans.push(Span::styled("│ ", Style::default().fg(CatppuccinFrappe::TEAL)));
            },
            Event::End(Tag::BlockQuote) => {
                in_blockquote = false;
                spans.push(Span::raw("\n"));
            },
            Event::Start(Tag::Emphasis) => {
                current_style = current_style.add_modifier(Modifier::ITALIC);
            },
            Event::End(Tag::Emphasis) => {
                current_style = Style::default().fg(CatppuccinFrappe::TEXT);
                if in_blockquote {
                    current_style = current_style.fg(CatppuccinFrappe::TEAL);
                }
            },
            Event::Start(Tag::Strong) => {
                current_style = current_style.add_modifier(Modifier::BOLD);
            },
            Event::End(Tag::Strong) => {
                current_style = Style::default().fg(CatppuccinFrappe::TEXT);
                if in_blockquote {
                    current_style = current_style.fg(CatppuccinFrappe::TEAL);
                }
            },
            Event::Start(Tag::Strikethrough) => {
                current_style = current_style.add_modifier(Modifier::CROSSED_OUT);
            },
            Event::End(Tag::Strikethrough) => {
                current_style = Style::default().fg(CatppuccinFrappe::TEXT);
                if in_blockquote {
                    current_style = current_style.fg(CatppuccinFrappe::TEAL);
                }
            },
            Event::Start(Tag::List(ordered)) => {
                // Start of a list
                spans.push(Span::raw("\n"));
                if let Some(start_number) = ordered {
                    // Store the start number for ordered lists
                    spans.push(Span::styled(format!("{}. ", start_number), 
                        Style::default().fg(CatppuccinFrappe::LAVENDER)));
                }
            },
            Event::End(Tag::List(_)) => {
                // End of a list
                spans.push(Span::raw("\n"));
            },
            Event::Start(Tag::Item) => {
                // List item (bullet handled in List start for ordered lists)
                if !spans.last().map_or(false, |span| span.content.ends_with(". ")) {
                    spans.push(Span::styled(" • ", Style::default().fg(CatppuccinFrappe::LAVENDER)));
                }
            },
            Event::End(Tag::Item) => {
                spans.push(Span::raw("\n"));
            },
            Event::TaskListMarker(TaskListMarker { checked }) => {
                // Replace the bullet with a checkbox
                if let Some(last) = spans.last() {
                    if last.content == " • " {
                        spans.pop(); // Remove the bullet
                        let checkbox = if checked {
                            "[✓] "
                        } else {
                            "[ ] "
                        };
                        spans.push(Span::styled(checkbox, Style::default().fg(CatppuccinFrappe::LAVENDER)));
                    }
                }
            },
            Event::Start(Tag::CodeBlock(kind)) => {
                spans.push(Span::raw("\n"));
                
                // Add language info if available
                if let CodeBlockKind::Fenced(lang) = kind {
                    if !lang.is_empty() {
                        spans.push(Span::styled(
                            format!("```{}\n", lang),
                            Style::default().fg(CatppuccinFrappe::SUBTEXT0)
                        ));
                    }
                }
                
                current_style = Style::default()
                    .fg(CatppuccinFrappe::PEACH)
                    .bg(CatppuccinFrappe::SURFACE0);
            },
            Event::End(Tag::CodeBlock(_)) => {
                current_style = Style::default().fg(CatppuccinFrappe::TEXT);
                spans.push(Span::styled("\n```", Style::default().fg(CatppuccinFrappe::SUBTEXT0)));
                spans.push(Span::raw("\n"));
            },
            Event::Code(code) => {
                spans.push(Span::styled(
                    code.to_string(),
                    Style::default()
                        .fg(CatppuccinFrappe::PEACH)
                        .bg(CatppuccinFrappe::SURFACE0)
                ));
            },
            Event::Start(Tag::Link(link_type, url, title)) => {
                // For links, we'll show the URL in a different color
                match link_type {
                    LinkType::Inline | LinkType::Reference | LinkType::Shortcut | LinkType::Collapsed => {
                        spans.push(Span::styled("[", Style::default().fg(CatppuccinFrappe::PINK)));
                        // The link text will be handled by subsequent Text events
                    },
                    LinkType::Autolink | LinkType::Email => {
                        // For autolinks, we'll just show the URL directly
                        spans.push(Span::styled(url.to_string(), Style::default().fg(CatppuccinFrappe::PINK)));
                    },
                }
            },
            Event::End(Tag::Link(_, url, _)) => {
                spans.push(Span::styled(
                    format!("]({})", url),
                    Style::default().fg(CatppuccinFrappe::PINK)
                ));
            },
            Event::Start(Tag::Image(_, url, title)) => {
                // For images, we'll show [Image: alt text (url)]
                spans.push(Span::styled("[Image: ", Style::default().fg(CatppuccinFrappe::YELLOW)));
                // The alt text will be handled by subsequent Text events
            },
            Event::End(Tag::Image(_, url, _)) => {
                spans.push(Span::styled(
                    format!(" ({})]", url),
                    Style::default().fg(CatppuccinFrappe::YELLOW)
                ));
            },
            Event::Start(Tag::Table(_)) => {
                in_table = true;
                spans.push(Span::raw("\n"));
            },
            Event::End(Tag::Table(_)) => {
                in_table = false;
                spans.push(Span::raw("\n"));
            },
            Event::Start(Tag::TableHead) => {
                // Table header styling
                current_style = Style::default()
                    .fg(CatppuccinFrappe::BLUE)
                    .add_modifier(Modifier::BOLD);
            },
            Event::End(Tag::TableHead) => {
                current_style = Style::default().fg(CatppuccinFrappe::TEXT);
                spans.push(Span::raw("\n"));
                // Add a separator line after the header
                spans.push(Span::styled("───────────────────", Style::default().fg(CatppuccinFrappe::SURFACE2)));
                spans.push(Span::raw("\n"));
            },
            Event::Start(Tag::TableRow) => {
                // Start a new row
                if !spans.last().map_or(false, |span| span.content.ends_with("\n")) {
                    spans.push(Span::raw("\n"));
                }
            },
            Event::End(Tag::TableRow) => {
                spans.push(Span::raw("\n"));
            },
            Event::Start(Tag::TableCell) => {
                // Add cell separator
                spans.push(Span::styled("| ", Style::default().fg(CatppuccinFrappe::SURFACE2)));
            },
            Event::End(Tag::TableCell) => {
                spans.push(Span::styled(" ", Style::default().fg(CatppuccinFrappe::SURFACE2)));
            },
            Event::Rule => {
                // Horizontal rule
                spans.push(Span::raw("\n"));
                spans.push(Span::styled("─────────────────────────────", 
                    Style::default().fg(CatppuccinFrappe::SURFACE2)));
                spans.push(Span::raw("\n"));
            },
            Event::SoftBreak => {
                spans.push(Span::raw(" "));
            },
            Event::HardBreak => {
                spans.push(Span::raw("\n"));
                if in_blockquote {
                    spans.push(Span::styled("│ ", Style::default().fg(CatppuccinFrappe::TEAL)));
                }
            },
            _ => {}
        }
    }
    
    spans
}

pub fn get_markdown_help() -> String {
    "Markdown Formatting:
# Heading 1
## Heading 2
### Heading 3

**Bold Text**
*Italic Text*
~~Strikethrough~~

- Unordered List Item
1. Ordered List Item
- [ ] Task (unchecked)
- [x] Task (checked)

> Blockquote text

`inline code`
```rust
// Code block with language
fn main() {
    println!(\"Hello, world!\");
}
```

[Link text](https://example.com)
![Image alt text](image-url)

Horizontal rule:
---

| Table | Header |
|-------|--------|
| Cell  | Cell   |
".to_string()
}
