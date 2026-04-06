use crate::app::{App, AppView, Mode};
use crate::panes::editor::EditorMode;
use crate::theme;

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    match app.mode {
        Mode::Command => render_command_line(f, app, area),
        _ => {
            if app.view == AppView::Edit && app.buffer.mode == EditorMode::Search {
                render_search_line(f, app, area);
            } else {
                render_input_area(f, app, area);
            }
        }
    }
}

fn render_input_area(f: &mut Frame, app: &App, area: Rect) {
    let prompt = Span::styled("  > ", theme::chat_user());
    let text = Span::styled(&app.input, theme::input_area());

    let line = Line::from(vec![prompt, text]);
    let widget = Paragraph::new(line).style(theme::input_area());
    f.render_widget(widget, area);

    if app.mode == Mode::Insert && app.view == AppView::Chat {
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

fn render_search_line(f: &mut Frame, app: &App, area: Rect) {
    let prefix = if app.buffer.search_input.starts_with('!') {
        Span::styled("/!", theme::chat_tool())
    } else {
        Span::styled("/", theme::statusline_mode_command())
    };
    let text = Span::styled(&app.buffer.search_input, theme::input_area());

    let line = Line::from(vec![prefix, text]);
    let widget = Paragraph::new(line).style(theme::input_area());
    f.render_widget(widget, area);

    let offset = if app.buffer.search_input.starts_with('!') { 2 } else { 1 };
    let x = area.x + offset + app.buffer.search_input.len() as u16;
    f.set_cursor_position((x, area.y));
}
