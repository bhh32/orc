use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal;

use std::io::{self, Write};

pub fn ask_permission(tool: &str, description: &str) -> io::Result<bool> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    write!(out, "\n  [{tool}] {description} (y/n): ")?;
    out.flush()?;

    terminal::enable_raw_mode()?;
    let result = loop {
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Char('y') | KeyCode::Char('Y') => break true,
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => break false,
                _ => continue,
            }
        }
    };
    terminal::disable_raw_mode()?;

    let answer = if result { "yes" } else { "no" };
    writeln!(out, "{answer}")?;

    Ok(result)
}
