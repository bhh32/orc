use crate::app::{App, Mode};
use crate::theme;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::horizontal([
        Constraint::Length(5),
        Constraint::Fill(1),
        Constraint::Length(30),
    ])
    .split(area);

    let mode_style = match app.mode {
        Mode::Normal => theme::statusline_mode_normal(),
        Mode::Insert => theme::statusline_mode_insert(),
        Mode::Command => theme::statusline_mode_command(),
    };
    let mode_label = format!(" {} ", app.mode.label());
    f.render_widget(Paragraph::new(Span::styled(mode_label, mode_style)), chunks[0]);

    let model = app.model.as_deref().unwrap_or("--");
    let center = format!("  {model}");
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
            .alignment(ratatui::layout::Alignment::Right),
        chunks[2],
    );
}
