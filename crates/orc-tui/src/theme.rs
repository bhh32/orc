use ratatui::style::{Color, Modifier, Style};

use std::sync::RwLock;
use std::sync::OnceLock;

pub struct Theme {
    pub base: Color,
    pub mantle: Color,
    pub crust: Color,
    pub surface0: Color,
    pub surface1: Color,
    pub surface2: Color,
    pub overlay0: Color,
    pub overlay1: Color,
    pub subtext0: Color,
    pub subtext1: Color,
    pub text: Color,
    pub lavender: Color,
    pub blue: Color,
    pub sapphire: Color,
    pub sky: Color,
    pub teal: Color,
    pub green: Color,
    pub yellow: Color,
    pub peach: Color,
    pub maroon: Color,
    pub red: Color,
    pub mauve: Color,
    pub pink: Color,
    pub flamingo: Color,
    pub rosewater: Color,
}

fn current_theme() -> &'static RwLock<Theme> {
    static CURRENT_THEME: OnceLock<RwLock<Theme>> = OnceLock::new();
    CURRENT_THEME.get_or_init(|| RwLock::new(catppuccin_mocha()))
}

fn catppuccin_mocha() -> Theme {
    Theme {
        base: Color::Rgb(30, 30, 46),
        mantle: Color::Rgb(24, 24, 37),
        crust: Color::Rgb(17, 17, 27),
        surface0: Color::Rgb(49, 50, 68),
        surface1: Color::Rgb(69, 71, 90),
        surface2: Color::Rgb(88, 91, 112),
        overlay0: Color::Rgb(108, 112, 134),
        overlay1: Color::Rgb(127, 132, 156),
        subtext0: Color::Rgb(166, 173, 200),
        subtext1: Color::Rgb(186, 194, 222),
        text: Color::Rgb(205, 214, 244),
        lavender: Color::Rgb(180, 190, 254),
        blue: Color::Rgb(137, 180, 250),
        sapphire: Color::Rgb(116, 199, 236),
        sky: Color::Rgb(137, 220, 235),
        teal: Color::Rgb(148, 226, 213),
        green: Color::Rgb(166, 227, 161),
        yellow: Color::Rgb(249, 226, 175),
        peach: Color::Rgb(250, 179, 135),
        maroon: Color::Rgb(235, 160, 172),
        red: Color::Rgb(243, 139, 168),
        mauve: Color::Rgb(203, 166, 247),
        pink: Color::Rgb(245, 194, 231),
        flamingo: Color::Rgb(242, 205, 205),
        rosewater: Color::Rgb(245, 224, 220),
    }
}

fn gruvbox_dark() -> Theme {
    Theme {
        base: Color::Rgb(40, 40, 40),
        mantle: Color::Rgb(29, 32, 33),
        crust: Color::Rgb(20, 20, 20),
        surface0: Color::Rgb(50, 48, 47),
        surface1: Color::Rgb(60, 56, 54),
        surface2: Color::Rgb(80, 73, 69),
        overlay0: Color::Rgb(124, 111, 100),
        overlay1: Color::Rgb(146, 131, 116),
        subtext0: Color::Rgb(189, 174, 147),
        subtext1: Color::Rgb(213, 196, 161),
        text: Color::Rgb(235, 219, 178),
        lavender: Color::Rgb(131, 165, 152),
        blue: Color::Rgb(69, 133, 136),
        sapphire: Color::Rgb(69, 133, 136),
        sky: Color::Rgb(131, 165, 152),
        teal: Color::Rgb(104, 157, 106),
        green: Color::Rgb(152, 151, 26),
        yellow: Color::Rgb(215, 153, 33),
        peach: Color::Rgb(214, 93, 14),
        maroon: Color::Rgb(204, 36, 29),
        red: Color::Rgb(204, 36, 29),
        mauve: Color::Rgb(177, 98, 134),
        pink: Color::Rgb(177, 98, 134),
        flamingo: Color::Rgb(214, 93, 14),
        rosewater: Color::Rgb(235, 219, 178),
    }
}

fn tokyo_night() -> Theme {
    Theme {
        base: Color::Rgb(26, 27, 38),
        mantle: Color::Rgb(22, 22, 30),
        crust: Color::Rgb(18, 18, 24),
        surface0: Color::Rgb(41, 46, 66),
        surface1: Color::Rgb(55, 60, 83),
        surface2: Color::Rgb(73, 79, 107),
        overlay0: Color::Rgb(86, 95, 137),
        overlay1: Color::Rgb(109, 117, 151),
        subtext0: Color::Rgb(150, 157, 185),
        subtext1: Color::Rgb(169, 177, 214),
        text: Color::Rgb(192, 202, 245),
        lavender: Color::Rgb(122, 162, 247),
        blue: Color::Rgb(122, 162, 247),
        sapphire: Color::Rgb(125, 207, 255),
        sky: Color::Rgb(125, 207, 255),
        teal: Color::Rgb(115, 218, 202),
        green: Color::Rgb(158, 206, 106),
        yellow: Color::Rgb(224, 175, 104),
        peach: Color::Rgb(255, 158, 100),
        maroon: Color::Rgb(247, 118, 142),
        red: Color::Rgb(247, 118, 142),
        mauve: Color::Rgb(187, 154, 247),
        pink: Color::Rgb(187, 154, 247),
        flamingo: Color::Rgb(255, 158, 100),
        rosewater: Color::Rgb(192, 202, 245),
    }
}

pub fn set_theme(name: &str) -> bool {
    let theme = match name {
        "catppuccin_mocha" | "catppuccin" => catppuccin_mocha(),
        "gruvbox_dark" | "gruvbox" => gruvbox_dark(),
        "tokyo_night" | "tokyo" => tokyo_night(),
        _ => return false,
    };
    *current_theme().write().unwrap() = theme;
    true
}

pub fn available_themes() -> Vec<&'static str> {
    vec!["catppuccin_mocha", "gruvbox_dark", "tokyo_night"]
}

pub fn base() -> Color {
    current_theme().read().unwrap().base
}

pub fn statusline() -> Style {
    let t = current_theme().read().unwrap();
    Style::default().fg(t.subtext1).bg(t.mantle)
}

pub fn statusline_mode_normal() -> Style {
    let t = current_theme().read().unwrap();
    Style::default().fg(t.crust).bg(t.blue).add_modifier(Modifier::BOLD)
}

pub fn statusline_mode_insert() -> Style {
    let t = current_theme().read().unwrap();
    Style::default().fg(t.crust).bg(t.green).add_modifier(Modifier::BOLD)
}

pub fn statusline_mode_command() -> Style {
    let t = current_theme().read().unwrap();
    Style::default().fg(t.crust).bg(t.mauve).add_modifier(Modifier::BOLD)
}

pub fn chat_user() -> Style {
    let t = current_theme().read().unwrap();
    Style::default().fg(t.blue).add_modifier(Modifier::BOLD)
}

pub fn chat_assistant() -> Style {
    let t = current_theme().read().unwrap();
    Style::default().fg(t.text)
}

pub fn chat_tool() -> Style {
    let t = current_theme().read().unwrap();
    Style::default().fg(t.teal)
}

pub fn chat_error() -> Style {
    let t = current_theme().read().unwrap();
    Style::default().fg(t.red)
}

pub fn chat_dim() -> Style {
    let t = current_theme().read().unwrap();
    Style::default().fg(t.overlay0)
}

pub fn border_focused() -> Style {
    let t = current_theme().read().unwrap();
    Style::default().fg(t.blue)
}

pub fn border_unfocused() -> Style {
    let t = current_theme().read().unwrap();
    Style::default().fg(t.surface1)
}

pub fn input_area() -> Style {
    let t = current_theme().read().unwrap();
    Style::default().fg(t.text).bg(t.mantle)
}

pub fn gutter() -> Style {
    let t = current_theme().read().unwrap();
    Style::default().fg(t.surface2)
}

pub fn selection() -> Style {
    let t = current_theme().read().unwrap();
    Style::default().bg(t.surface1)
}
