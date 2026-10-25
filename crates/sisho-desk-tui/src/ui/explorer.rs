//! Explorer tab rendering: workspace file tree + syntax-highlighted
//! file preview / inline editor pane.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use super::highlight::highlight_source_file;
use sisho_desk_core::app::{App, FocusedPane};

pub(super) fn render_explorer_view(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let list_height = chunks[0].height.saturating_sub(2) as usize;
    let preview_height = chunks[1].height.saturating_sub(2) as usize;

    app.adjust_tree_scrolloff(list_height);
    app.adjust_preview_scrolloff(preview_height);

    let vis_indices = app.visible_tree_indices();
    let visible_height = chunks[0].height.saturating_sub(2) as usize;
    let visible_items: Vec<ListItem> = vis_indices
        .iter()
        .skip(app.tree_scroll_offset)
        .take(visible_height)
        .enumerate()
        .map(|(i, &real_idx)| {
            let vis_idx = app.tree_scroll_offset + i;
            let node = &app.tree_nodes[real_idx];
            let indent = "  ".repeat(node.depth);

            let ext = node.path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let line_spans = if node.is_dir {
                let icon = if node.expanded {
                    "[-] 📁 "
                } else {
                    "[+] 📁 "
                };
                vec![
                    Span::raw(indent),
                    Span::styled(icon, Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{}/", node.name),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]
            } else if ext.eq_ignore_ascii_case("ctmt") {
                vec![
                    Span::raw(indent),
                    Span::styled("📦 ", Style::default().fg(Color::Yellow)),
                    Span::styled(&node.name, Style::default().fg(Color::Yellow)),
                ]
            } else if ext.eq_ignore_ascii_case("tmt") || ext.eq_ignore_ascii_case("tm") {
                vec![
                    Span::raw(indent),
                    Span::styled("🔷 ", Style::default().fg(Color::Cyan)),
                    Span::styled(&node.name, Style::default().fg(Color::White)),
                ]
            } else if node.migration_item.is_some() || ext.eq_ignore_ascii_case("md") {
                let converted = app
                    .migration_items
                    .iter()
                    .find(|i| i.source_path == node.path)
                    .is_some_and(|i| i.converted);
                let status = if converted { " (DONE)" } else { "" };
                vec![
                    Span::raw(indent),
                    Span::styled("📝 ", Style::default().fg(Color::Green)),
                    Span::styled(&node.name, Style::default().fg(Color::White)),
                    Span::styled(status, Style::default().fg(Color::DarkGray)),
                ]
            } else {
                vec![
                    Span::raw(indent),
                    Span::styled("📄 ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&node.name, Style::default().fg(Color::DarkGray)),
                ]
            };

            let style = if vis_idx == app.tree_index {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(line_spans)).style(style)
        })
        .collect();

    let tree_title = if !app.tree_filter_input.is_empty() {
        format!(
            " Workspace Explorer (Filter: \"{}\" - {}/{} items) ",
            app.tree_filter_input,
            vis_indices.len(),
            app.tree_nodes.len()
        )
    } else {
        " Workspace File Explorer (Press [/] to Filter) ".to_string()
    };

    let file_list = List::new(visible_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(super::get_border_style(
                app.focused_pane == FocusedPane::List,
            ))
            .title(tree_title),
    );
    f.render_widget(file_list, chunks[0]);

    // Preview Pane
    let mut preview_lines = Vec::new();
    let mut title =
        " File Content Preview (Press [e] to Edit, [E] Open External $EDITOR) ".to_string();

    if let Some(editor) = &app.inline_editor {
        let dirty = if editor.is_dirty { " [*MODIFIED*]" } else { "" };
        let file_name = editor
            .file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        title = format!(
            " EDITING: {file_name} [Ln {}, Col {}]{dirty} (Ctrl+S: Save, Esc: Exit) ",
            editor.cursor_row + 1,
            editor.cursor_col + 1
        );

        let full_src = editor.lines.join("\n");
        let highlighted = highlight_source_file(&editor.file_path, &full_src);
        let total_lines = highlighted.len();
        let num_width = total_lines.to_string().len().max(2);

        for (row_idx, line) in highlighted.into_iter().enumerate() {
            let line_num_span = super::line_number_span(row_idx, num_width);

            if row_idx == editor.cursor_row {
                let mut cursor_line_spans = vec![line_num_span];
                let mut char_count = 0;
                let mut cursor_rendered = false;

                for span in line.spans {
                    let text = span.content.to_string();
                    let span_len = text.chars().count();

                    if !cursor_rendered
                        && editor.cursor_col >= char_count
                        && editor.cursor_col < char_count + span_len
                    {
                        let offset = editor.cursor_col - char_count;
                        let byte_offset = text
                            .char_indices()
                            .nth(offset)
                            .map(|(i, _)| i)
                            .unwrap_or_else(|| text.len());
                        let (before, rest) = text.split_at(byte_offset);
                        if !before.is_empty() {
                            cursor_line_spans.push(Span::styled(before.to_string(), span.style));
                        }

                        let mut rest_chars = rest.chars();
                        if let Some(c_char) = rest_chars.next() {
                            cursor_line_spans.push(Span::styled(
                                c_char.to_string(),
                                Style::default()
                                    .bg(Color::Yellow)
                                    .fg(Color::Black)
                                    .add_modifier(Modifier::BOLD),
                            ));
                            let remaining: String = rest_chars.collect();
                            if !remaining.is_empty() {
                                cursor_line_spans.push(Span::styled(remaining, span.style));
                            }
                        } else {
                            cursor_line_spans.push(Span::styled(
                                " ",
                                Style::default()
                                    .bg(Color::Yellow)
                                    .fg(Color::Black)
                                    .add_modifier(Modifier::BOLD),
                            ));
                        }
                        cursor_rendered = true;
                    } else {
                        cursor_line_spans.push(span);
                    }
                    char_count += span_len;
                }

                if !cursor_rendered {
                    cursor_line_spans.push(Span::styled(
                        " ",
                        Style::default()
                            .bg(Color::Yellow)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
                preview_lines.push(Line::from(cursor_line_spans));
            } else {
                let mut spans = vec![line_num_span];
                spans.extend(line.spans);
                preview_lines.push(Line::from(spans));
            }
        }
    } else if let Some(&real_idx) = vis_indices.get(app.tree_index) {
        if let Some(node) = app.tree_nodes.get(real_idx) {
            if node.is_dir {
                title = format!(" Directory: {}/ ", node.name);
                preview_lines.push(Line::from(""));
                preview_lines.push(Line::from(Span::styled(
                    format!("  📁 Directory: {}", node.path.display()),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
                preview_lines.push(Line::from(""));
                preview_lines.push(Line::from(Span::styled(
                    "  Press [Enter] or [Space] to expand/collapse directory in the tree.",
                    Style::default().fg(Color::DarkGray),
                )));
            } else {
                let file_size_str = std::fs::metadata(&node.path)
                    .map(|m| format_file_size(m.len()))
                    .unwrap_or_else(|_| "0 B".to_string());

                match std::fs::read_to_string(&node.path) {
                    Ok(src) => {
                        let highlighted = highlight_source_file(&node.path, &src);
                        let total_lines = highlighted.len();
                        let num_width = total_lines.to_string().len().max(2);

                        title = format!(
                            " Preview: {} ({total_lines} lines, {file_size_str}) ",
                            node.name
                        );

                        for (row_idx, line) in highlighted.into_iter().enumerate() {
                            let line_num_span = super::line_number_span(row_idx, num_width);
                            let mut spans = vec![line_num_span];
                            spans.extend(line.spans);
                            preview_lines.push(Line::from(spans));
                        }
                    }
                    Err(_) => {
                        title = format!(" Preview: {} ({file_size_str}) ", node.name);
                        preview_lines.push(Line::from(""));
                        preview_lines.push(Line::from(Span::styled(
                            "  (Binary or unreadable file)",
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                }
            }
        }
    } else {
        preview_lines.push(Line::from("No files found in workspace directory."));
    }

    let preview = Paragraph::new(preview_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(super::get_border_style(
                    app.focused_pane == FocusedPane::Preview,
                ))
                .title(title),
        )
        .scroll((app.preview_scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(preview, chunks[1]);
}

fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

pub(super) fn explorer_controls_info(app: &App) -> Line<'static> {
    if app.is_filtering_tree {
        Line::from(vec![
            Span::styled(
                " SEARCH FILTER: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("\"{}\" ", app.tree_filter_input),
                Style::default().fg(Color::Cyan),
            ),
            super::key("[Enter] "),
            Span::raw("Apply  "),
            super::key("[Esc] "),
            Span::raw("Clear & Exit"),
        ])
    } else if app.inline_editor.is_some() {
        Line::from(vec![
            super::key("[Ctrl+S] "),
            Span::raw("Save  "),
            super::key("[Esc] "),
            Span::raw("Exit Edit  "),
            super::key("[Arrows] "),
            Span::raw("Navigate  "),
            super::key("[Enter] "),
            Span::raw("Newline"),
        ])
    } else if app.focused_pane == FocusedPane::Preview {
        Line::from(vec![
            super::key("[j/k / ↑/↓] "),
            Span::raw("Scroll  "),
            super::key("[e] "),
            Span::raw("Edit  "),
            super::key("[E] "),
            Span::raw("$EDITOR  "),
            super::key("[Esc] "),
            Span::raw("Back to Tree  "),
            super::key("[?] "),
            Span::raw("Help"),
        ])
    } else if !app.tree_filter_input.is_empty() {
        Line::from(vec![
            Span::styled(
                format!(" Filter: \"{}\"  ", app.tree_filter_input),
                Style::default().fg(Color::Cyan),
            ),
            super::key("[/] "),
            Span::raw("Edit Filter  "),
            super::key("[Esc] "),
            Span::raw("Clear Filter  "),
            super::key("[e] "),
            Span::raw("Edit  "),
            super::key("[?] "),
            Span::raw("Help"),
        ])
    } else {
        Line::from(vec![
            super::key("[j/k] "),
            Span::raw("Move  "),
            super::key("[Enter/Space] "),
            Span::raw("Toggle  "),
            super::key("[e] "),
            Span::raw("Edit  "),
            super::key("[E] "),
            Span::raw("$EDITOR  "),
            super::key("[/] "),
            Span::raw("Filter  "),
            super::key("[Tab] "),
            Span::raw("Next tab  "),
            super::key("[?] "),
            Span::raw("Help"),
        ])
    }
}
