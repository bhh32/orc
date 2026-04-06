use crate::app::{App, AppView, Mode};
use crate::theme;

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::horizontal([
        Constraint::Length(5),
        Constraint::Fill(1),
        Constraint::Length(35),
    ])
    .split(area);

    let mode_label = active_mode_label(app);
    let mode_style = match mode_label {
        "NOR" => theme::statusline_mode_normal(),
        "INS" => theme::statusline_mode_insert(),
        "SEL" => theme::statusline_mode_command(),
        "CMD" => theme::statusline_mode_command(),
        _ => theme::statusline(),
    };
    f.render_widget(
        Paragraph::new(Span::styled(format!(" {mode_label} "), mode_style)),
        chunks[0],
    );

    let center = match app.view {
        AppView::Chat => {
            let model = app.model.as_deref().unwrap_or("--");
            format!("  {model}")
        }
        AppView::Edit => {
            let name = app.buffer.file_name();
            let modified = if app.buffer.modified { " [+]" } else { "" };
            let line = app.buffer.cursor_line() + 1;
            let col = app.buffer.cursor_col() + 1;
            format!("  {name}{modified}  {line}:{col}")
        }
    };
    f.render_widget(
        Paragraph::new(Span::styled(center, theme::statusline())),
        chunks[1],
    );

    let right = format!(
        "{}↑ {}↓  ${:.2}  T{}  ",
        app.input_tokens, app.output_tokens, app.cost_usd, app.turns,
    );
    f.render_widget(
        Paragraph::new(Span::styled(right, theme::statusline()))
            .alignment(Alignment::Right),
        chunks[2],
    );
}

fn active_mode_label(app: &App) -> &str {
    if app.mode == Mode::Command {
        return "CMD";
    }

    match app.view {
        AppView::Chat => app.mode.label(),
        AppView::Edit => app.buffer.mode.label(),
    }
}
