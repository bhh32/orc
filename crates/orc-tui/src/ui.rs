use crate::app::App;
use crate::panes::{chat, input, status};
use crate::theme;

use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::Block;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let bg = Block::default().style(ratatui::style::Style::default().bg(theme::BASE));
    f.render_widget(bg, area);

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
