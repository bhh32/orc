use crate::app::{App, Mode};
use crate::theme;

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    match app.mode {
        Mode::Command => render_command_line(f, app, area),
        _ => render_input_area(f, app, area),
    }
}

fn render_input_area(f: &mut Frame, app: &App, area: Rect) {
    let prompt = Span::styled("  > ", theme::chat_user());
    let text = Span::styled(&app.input, theme::input_area());

    let line = Line::from(vec![prompt, text]);
    let widget = Paragraph::new(line).style(theme::input_area());
    f.render_widget(widget, area);

    if app.mode == Mode::Insert {
        let x = area.x + 4 + app.input_cursor as u16;
        let y = area.y;
        f.set_cursor_position((x, y));
    }
}

fn render_command_line(f: &mut Frame, app: &App, area: Rect) {
    let prompt = Span::styled(":", theme::statusline_mode_command());
    let text = Span::styled(&app.command_input, theme::input_area());

    let line = Line::from(vec![prompt, text]);
    let widget = Paragraph::new(line).style(theme::input_area());
    f.render_widget(widget, area);

    let x = area.x + 1 + app.command_cursor as u16;
    let y = area.y;
    f.set_cursor_position((x, y));
}
