use crate::app::{App, AppView, EditFocus};
use crate::panes::{chat, editor, input, sidebar, status};
use crate::theme;

use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::Block;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let bg = Block::default().style(ratatui::style::Style::default().bg(theme::BASE));
    f.render_widget(bg, area);

    match app.view {
        AppView::Chat => render_chat_view(f, app),
        AppView::Edit => render_edit_view(f, app),
    }
}

fn render_chat_view(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    chat::render(f, app, chunks[0]);
    status::render(f, app, chunks[1]);
    input::render(f, app, chunks[2]);
}

fn render_edit_view(f: &mut Frame, app: &App) {
    let area = f.area();

    let main = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .split(area);

    let cols = Layout::horizontal([
        Constraint::Length(22),
        Constraint::Fill(1),
    ])
    .split(main[0]);

    let right = Layout::vertical([
        Constraint::Fill(3),
        Constraint::Fill(1),
    ])
    .split(cols[1]);

    sidebar::render(f, &app.sidebar, cols[0], app.edit_focus == EditFocus::Sidebar);
    editor::render(f, &app.buffer, right[0], app.edit_focus == EditFocus::Editor);
    chat::render(f, app, right[1]);
    status::render(f, app, main[1]);
}
