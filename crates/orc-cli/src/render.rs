use crossterm::style::{Attribute, Color, SetAttribute, SetForegroundColor, ResetColor};

use std::io::{self, Write};

pub struct Renderer {
    stdout: io::Stdout,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            stdout: io::stdout(),
        }
    }

    pub fn print_assistant_text(&mut self, text: &str) {
        let mut out = self.stdout.lock();
        let _ = write!(out, "{text}");
        let _ = out.flush();
    }

    pub fn print_tool_start(&mut self, name: &str, _id: &str) {
        let mut out = self.stdout.lock();
        let _ = write!(
            out,
            "\n{color}  [{name}]{reset} ",
            color = SetForegroundColor(Color::DarkCyan),
            reset = ResetColor,
        );
        let _ = out.flush();
    }

    pub fn print_tool_result(&mut self, output: &str, is_error: bool) {
        let mut out = self.stdout.lock();

        let color = if is_error {
            SetForegroundColor(Color::Red)
        } else {
            SetForegroundColor(Color::DarkGreen)
        };

        let lines: Vec<&str> = output.lines().collect();
        let preview = if lines.len() > 10 {
            let mut buf = String::new();
            for line in &lines[..10] {
                buf.push_str("    ");
                buf.push_str(line);
                buf.push('\n');
            }
            buf.push_str(&format!("    ... ({} more lines)", lines.len() - 10));
            buf
        } else {
            output.to_string()
        };

        let _ = writeln!(out, "{color}{preview}{reset}", reset = ResetColor);
        let _ = out.flush();
    }

    pub fn print_error(&mut self, msg: &str) {
        let mut out = self.stdout.lock();
        let _ = writeln!(
            out,
            "{color}error: {msg}{reset}",
            color = SetForegroundColor(Color::Red),
            reset = ResetColor,
        );
        let _ = out.flush();
    }

    pub fn print_status(&mut self, msg: &str) {
        let mut out = self.stdout.lock();
        let _ = writeln!(
            out,
            "{dim}{msg}{reset}",
            dim = SetAttribute(Attribute::Dim),
            reset = SetAttribute(Attribute::Reset),
        );
        let _ = out.flush();
    }

    pub fn newline(&mut self) {
        let mut out = self.stdout.lock();
        let _ = writeln!(out);
        let _ = out.flush();
    }
}
