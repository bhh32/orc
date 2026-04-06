use crate::app::{App, ChatRole};
use crate::theme;

use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::NONE);

    let mut lines: Vec<Line> = Vec::new();

    for msg in &app.messages {
        match &msg.role {
            ChatRole::User => {
                lines.push(Line::from(vec![
                    Span::styled("  > ", theme::chat_user()),
                    Span::styled(&msg.content, theme::chat_user()),
                ]));
                lines.push(Line::from(""));
            }
            ChatRole::Assistant => {
                for text_line in msg.content.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("  {text_line}"),
                        theme::chat_assistant(),
                    )));
                }
                lines.push(Line::from(""));
            }
            ChatRole::ToolUse { name, .. } => {
                let tool_style = theme::chat_tool();
                let bold_tool = tool_style.add_modifier(Modifier::BOLD);
                lines.push(Line::from(vec![
                    Span::styled("  ╭─ ", theme::chat_dim()),
                    Span::styled(name, bold_tool),
                    Span::styled(" ", tool_style),
                    Span::styled(&msg.content, tool_style),
                ]));
            }
            ChatRole::ToolResult { is_error } => {
                let content = &msg.content;
                if content.is_empty() {
                    lines.push(Line::from(Span::styled("  ╰─ done", theme::chat_dim())));
                } else if *is_error {
                    for result_line in content.lines().take(5) {
                        lines.push(Line::from(vec![
                            Span::styled("  │  ", theme::chat_dim()),
                            Span::styled(result_line, theme::chat_error()),
                        ]));
                    }
                    lines.push(Line::from(Span::styled("  ╰─ error", theme::chat_error())));
                } else {
                    let total = content.lines().count();
                    for result_line in content.lines().take(8) {
                        lines.push(Line::from(vec![
                            Span::styled("  │  ", theme::chat_dim()),
                            Span::styled(result_line, theme::chat_dim()),
                        ]));
                    }
                    if total > 8 {
                        lines.push(Line::from(Span::styled(
                            format!("  │  ... ({} more lines)", total - 8),
                            theme::chat_dim(),
                        )));
                    }
                    lines.push(Line::from(Span::styled("  ╰─ done", theme::chat_dim())));
                }
            }
            ChatRole::System => {
                lines.push(Line::from(Span::styled(
                    format!("  {}", msg.content),
                    theme::chat_dim(),
                )));
            }
            ChatRole::Error => {
                lines.push(Line::from(Span::styled(
                    format!("  error: {}", msg.content),
                    theme::chat_error(),
                )));
            }
        }
    }

    if app.streaming {
        lines.push(Line::from(Span::styled("  ...", theme::chat_dim())));
    }

    let total_lines = lines.len() as u16;
    let visible = area.height.saturating_sub(1);
    let max_scroll = total_lines.saturating_sub(visible);

    // Use user's scroll offset if set, otherwise auto-scroll to bottom
    let scroll = if app.scroll_offset == u16::MAX || app.scroll_offset >= max_scroll {
        max_scroll
    } else {
        app.scroll_offset.min(max_scroll)
    };

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    f.render_widget(paragraph, area);
}
