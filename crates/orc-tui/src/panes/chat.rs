use crate::app::{App, ChatRole};
use crate::theme;

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::NONE);

    let mut lines: Vec<Line> = Vec::new();

    for msg in &app.messages {
        match msg.role {
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
            ChatRole::Tool => {
                lines.push(Line::from(Span::styled(
                    format!("  {}", msg.content),
                    theme::chat_tool(),
                )));
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
    let scroll = if total_lines > visible {
        total_lines - visible
    } else {
        0
    };

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    f.render_widget(paragraph, area);
}
