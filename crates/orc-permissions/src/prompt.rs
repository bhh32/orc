use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal;

use std::io::{self, Write};

pub fn ask_permission(tool: &str, description: &str) -> io::Result<bool> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    write!(out, "\n  [{tool}] {description} (y/n): ")?;
    out.flush()?;

    terminal::enable_raw_mode()?;
    let result = read_yes_no();
    terminal::disable_raw_mode()?;

    let accepted = match result {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let answer = if accepted { "yes" } else { "no" };
    writeln!(out, "{answer}")?;

    Ok(accepted)
}

fn read_yes_no() -> io::Result<bool> {
    loop {
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(true),
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => return Ok(false),
                _ => continue,
            }
        }
    }
}
