use ratatui::style::{Color, Modifier, Style};

// Catppuccin Mocha palette
pub const BASE: Color = Color::Rgb(30, 30, 46);        // #1e1e2e
pub const MANTLE: Color = Color::Rgb(24, 24, 37);      // #181825
pub const CRUST: Color = Color::Rgb(17, 17, 27);       // #11111b
pub const SURFACE0: Color = Color::Rgb(49, 50, 68);    // #313244
pub const SURFACE1: Color = Color::Rgb(69, 71, 90);    // #45475a
pub const SURFACE2: Color = Color::Rgb(88, 91, 112);   // #585b70
pub const OVERLAY0: Color = Color::Rgb(108, 112, 134); // #6c7086
pub const OVERLAY1: Color = Color::Rgb(127, 132, 156); // #7f849c
pub const SUBTEXT0: Color = Color::Rgb(166, 173, 200); // #a6adc8
pub const SUBTEXT1: Color = Color::Rgb(186, 194, 222); // #bac2de
pub const TEXT: Color = Color::Rgb(205, 214, 244);      // #cdd6f4
pub const LAVENDER: Color = Color::Rgb(180, 190, 254);  // #b4befe
pub const BLUE: Color = Color::Rgb(137, 180, 250);      // #89b4fa
pub const SAPPHIRE: Color = Color::Rgb(116, 199, 236);  // #74c7ec
pub const SKY: Color = Color::Rgb(137, 220, 235);       // #89dceb
pub const TEAL: Color = Color::Rgb(148, 226, 213);      // #94e2d5
pub const GREEN: Color = Color::Rgb(166, 227, 161);     // #a6e3a1
pub const YELLOW: Color = Color::Rgb(249, 226, 175);    // #f9e2af
pub const PEACH: Color = Color::Rgb(250, 179, 135);     // #fab387
pub const MAROON: Color = Color::Rgb(235, 160, 172);    // #eba0ac
pub const RED: Color = Color::Rgb(243, 139, 168);       // #f38ba8
pub const MAUVE: Color = Color::Rgb(203, 166, 247);     // #cba6f7
pub const PINK: Color = Color::Rgb(245, 194, 231);      // #f5c2e7
pub const FLAMINGO: Color = Color::Rgb(242, 205, 205);  // #f2cdcd
pub const ROSEWATER: Color = Color::Rgb(245, 224, 220); // #f5e0dc

pub fn statusline() -> Style {
    Style::default().fg(SUBTEXT1).bg(MANTLE)
}

pub fn statusline_mode_normal() -> Style {
    Style::default().fg(CRUST).bg(BLUE).add_modifier(Modifier::BOLD)
}

pub fn statusline_mode_insert() -> Style {
    Style::default().fg(CRUST).bg(GREEN).add_modifier(Modifier::BOLD)
}

pub fn statusline_mode_command() -> Style {
    Style::default().fg(CRUST).bg(MAUVE).add_modifier(Modifier::BOLD)
}

pub fn chat_user() -> Style {
    Style::default().fg(BLUE).add_modifier(Modifier::BOLD)
}

pub fn chat_assistant() -> Style {
    Style::default().fg(TEXT)
}

pub fn chat_tool() -> Style {
    Style::default().fg(TEAL)
}

pub fn chat_error() -> Style {
    Style::default().fg(RED)
}

pub fn chat_dim() -> Style {
    Style::default().fg(OVERLAY0)
}

pub fn border_focused() -> Style {
    Style::default().fg(BLUE)
}

pub fn border_unfocused() -> Style {
    Style::default().fg(SURFACE1)
}

pub fn input_area() -> Style {
    Style::default().fg(TEXT).bg(MANTLE)
}

pub fn gutter() -> Style {
    Style::default().fg(SURFACE2)
}

pub fn selection() -> Style {
    Style::default().bg(SURFACE1)
}
