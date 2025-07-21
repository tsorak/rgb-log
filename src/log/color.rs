use crossterm::style::{self, StyledContent, Stylize, style};

pub trait GetColor: 'static {
    fn get_colored_str(&self) -> StyledContent<&'static str>;
    fn get_inner_str(&self) -> &'static str;
}

#[derive(Clone)]
pub enum Color {
    Red(&'static str),
    Green(&'static str),
    Blue(&'static str),
    Cyan(&'static str),
    Yellow(&'static str),
    Magenta(&'static str),
}

impl GetColor for &'static str {
    fn get_colored_str(&self) -> StyledContent<&'static str> {
        style(*self)
    }

    fn get_inner_str(&self) -> &'static str {
        self
    }
}

impl GetColor for Color {
    fn get_colored_str(&self) -> StyledContent<&'static str> {
        match self {
            Color::Red(s) => style(*s).with(style::Color::Red),
            Color::Green(s) => style(*s).with(style::Color::Green),
            Color::Blue(s) => style(*s).with(style::Color::Blue),
            Color::Cyan(s) => style(*s).with(style::Color::Cyan),
            Color::Yellow(s) => style(*s).with(style::Color::Yellow),
            Color::Magenta(s) => style(*s).with(style::Color::Magenta),
        }
    }

    fn get_inner_str(&self) -> &'static str {
        match self {
            Color::Red(s) => s,
            Color::Green(s) => s,
            Color::Blue(s) => s,
            Color::Cyan(s) => s,
            Color::Yellow(s) => s,
            Color::Magenta(s) => s,
        }
    }
}
