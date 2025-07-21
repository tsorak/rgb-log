use std::{collections::HashMap, sync::Arc};

use crossterm::style::StyledContent;

use padding::PadLeft;

use crate::{buf::LogBuffer, color::Color};

mod builder;
pub use builder::Builder as LogBuilder;

mod program_name;
pub use program_name::ProgramName;

type S = &'static str;

pub const DEFAULT_LEVELS: [Color; 4] = [
    Color::Blue("INFO"),
    Color::Green("OK"),
    Color::Red("ERROR"),
    Color::Cyan("DEBUG"),
];

pub struct Log {
    buf: Box<dyn LogBuffer>,
    program_name: Option<String>,
    submodule_pad: PadLeft<'static>,
    level_pad: PadLeft<'static>,
    level_color: HashMap<&'static str, StyledContent<&'static str>>,
}

impl<'b> Log {
    pub fn new<I1, I2>(buffer: impl LogBuffer, submodule_names: I1, levels: I2) -> Arc<Self>
    where
        I1: IntoIterator<Item = S>,
        I2: IntoIterator<Item = (S, StyledContent<S>)> + Clone,
    {
        let level_names = levels.clone().into_iter().map(|(s, _)| s);

        Self {
            buf: Box::new(buffer),
            program_name: ProgramName::CrateName.try_to_string(),
            submodule_pad: PadLeft::new(submodule_names),
            level_pad: PadLeft::new(level_names),
            level_color: levels.into_iter().collect(),
        }
        .into()
    }

    pub fn submodule(self: &Arc<Self>, name: &'static str) -> SubmoduleLog {
        SubmoduleLog::new(self.clone(), name)
    }

    pub fn debug<L: Loggable>(&'b self, content: L) {
        Print::new(self, (None, Some("DEBUG"), Some(content))).print();
    }

    pub fn info<L: Loggable>(&'b self, content: L) {
        Print::new(self, (None, Some("INFO"), Some(content))).print();
    }

    pub fn ok<L: Loggable>(&'b self, content: L) {
        Print::new(self, (None, Some("OK"), Some(content))).print();
    }

    pub fn error<L: Loggable>(&'b self, content: L) {
        Print::new(self, (None, Some("ERROR"), Some(content))).print();
    }

    //pub async fn get_buf(&self) -> RwLockReadGuard<'_, Vec<String>> {
    //    self.buf.read().await
    //}
}

#[derive(Clone)]
pub struct SubmoduleLog {
    log: Arc<Log>,
    submod: &'static str,
}

impl SubmoduleLog {
    pub fn new(log: Arc<Log>, submod: &'static str) -> Self {
        Self { log, submod }
    }

    pub fn debug<L: Loggable>(&self, content: L) {
        Print::new(&self.log, (Some(self.submod), Some("DEBUG"), Some(content))).print();
    }

    pub fn info<L: Loggable>(&self, content: L) {
        Print::new(&self.log, (Some(self.submod), Some("INFO"), Some(content))).print();
    }

    pub fn ok<L: Loggable>(&self, content: L) {
        Print::new(&self.log, (Some(self.submod), Some("OK"), Some(content))).print();
    }

    pub fn error<L: Loggable>(&self, content: L) {
        Print::new(&self.log, (Some(self.submod), Some("ERROR"), Some(content))).print();
    }
}

pub struct Print<'a, 'b, L: Loggable> {
    log: &'b Log,
    submod: Option<&'a str>,
    level: Option<&'a str>,
    content: Option<L>,
}

impl<'a, 'b, L: Loggable> Print<'a, 'b, L> {
    pub fn new(
        log: &'b Log,
        (submod, level, content): (Option<&'a str>, Option<&'a str>, Option<L>),
    ) -> Self {
        Self {
            log,
            submod,
            level,
            content,
        }
    }

    pub fn submod(&mut self, submod: &'a str) -> &mut Self {
        let _ = self.submod.insert(submod);
        self
    }

    pub fn level(&mut self, level: &'a str) -> &mut Self {
        let _ = self.level.insert(level);
        self
    }

    pub fn content(&mut self, content: L) -> &mut Self {
        let _ = self.content.insert(content);
        self
    }

    // Output methods

    pub fn debug(mut self, content: L) {
        let _ = self.level.insert("DEBUG");
        let _ = self.content.insert(content);
        Self::print(self);
    }

    pub fn info(mut self, content: L) {
        let _ = self.level.insert("INFO");
        let _ = self.content.insert(content);
        Self::print(self);
    }

    pub fn ok(mut self, content: L) {
        let _ = self.level.insert("OK");
        let _ = self.content.insert(content);
        Self::print(self);
    }

    pub fn error(mut self, content: L) {
        let _ = self.level.insert("ERROR");
        let _ = self.content.insert(content);
        Self::print(self);
    }

    pub fn printc(mut self, content: L) {
        let _ = self.content.insert(content);
        Self::print(self);
    }

    pub fn print(self) {
        let line = self.get_line();
        print!("{line}");

        self.log.buf.push_line(line);
    }

    pub fn into_line(self) -> String {
        let line = self.get_line();
        print!("{line}");

        self.log.buf.push_line(line.clone());

        line
    }

    // priv

    #[allow(unreachable_code)]
    fn get_line(&self) -> String {
        let program_and_modpart = if let Some(ref program) = self.log.program_name {
            match self.submod {
                Some(submod) => {
                    format!("[{program} {}]", self.log.submodule_pad.get(submod))
                }
                None => {
                    let spacing = " ".repeat(self.log.submodule_pad.width.into());
                    format!("[{program}] {spacing}")
                }
            }
        } else {
            match self.submod {
                Some(submod) => {
                    format!("[{}]", self.log.submodule_pad.get(submod))
                }
                None => {
                    let spacing = " ".repeat(self.log.submodule_pad.width.into());
                    format!("{spacing}")
                }
            }
        };

        let (padding, level) = self.log.level_pad.get_split(self.level.unwrap_or("DEBUG"));
        let level = self
            .log
            .level_color
            .get(level)
            .map(|s| format!("{padding}{s}"))
            .unwrap_or(format!("{padding}{level}"));

        let content = self.get_content();

        #[cfg(feature = "chrono")]
        {
            use chrono::{SecondsFormat, Utc};

            #[cfg(feature = "seconds")]
            let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
            #[cfg(feature = "milliseconds")]
            let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

            #[cfg(feature = "rfc")]
            {
                return format!("{timestamp} {program_and_modpart} {level}: {content}\r\n");
            }

            let date_time = timestamp.replace("T", " ").replace("Z", "");
            return format!("{date_time} {program_and_modpart} {level}: {content}\r\n");
        }

        format!("{program_and_modpart} {level}: {content}\r\n")
    }

    fn get_content(&self) -> String {
        self.content
            .as_ref()
            .map_or(Default::default(), |l| l.as_loggable())
    }
}

pub trait Loggable: Sized {
    fn as_loggable(&self) -> String;
}

impl<T: ToString> Loggable for T {
    fn as_loggable(&self) -> String {
        self.to_string()
    }
}

pub mod color {
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
}

mod padding {
    use std::collections::HashMap;

    pub struct PadLeft<'a> {
        pub width: u8,
        pub map: HashMap<&'a str, u8>,
    }

    impl<'a> PadLeft<'a> {
        pub fn new<T>(iterable: T) -> Self
        where
            T: IntoIterator<Item = S<'a>>,
        {
            let (width, map) = to_pad_map_left(iterable);
            Self { width, map }
        }

        pub fn get(&self, k: &'a str) -> String {
            let (padding, key) = self.get_split(k);
            format!("{padding}{key}")
        }

        /// Get per key. If None, compute padding per widest known key
        pub fn get_split(&self, k: &'a str) -> (String, &'a str) {
            match self.map.get(k) {
                Some(n) => {
                    let padding = " ".repeat((*n).into());
                    (padding, k)
                }
                None => {
                    let letters_count = k.chars().count() as u8;

                    if letters_count < self.width {
                        let n = self.width - letters_count;
                        let s = " ".repeat(n.into());
                        (s, k)
                    } else {
                        (String::new(), k)
                    }
                }
            }
        }
    }

    type S<'a> = &'a str;

    /// v is k with left padding.
    /// Padding amount is the number of letters in the longest k letter-wise.
    fn to_pad_map_left<'a, T>(iterable: T) -> (u8, HashMap<&'a str, u8>)
    where
        T: IntoIterator<Item = S<'a>>,
    {
        let map: HashMap<&'a str, u8> = iterable
            .into_iter()
            .map(|s| {
                let letters_count = s.chars().count() as u8;

                (s, letters_count)
            })
            .collect();

        let mut widest = 0;
        for letters_count in map.values() {
            if widest < *letters_count {
                widest = *letters_count;
            }
        }

        // Set the padding that each one needs
        let pad_map = map
            .into_iter()
            .map(|(k, v)| {
                let needed_padding = widest - v;
                (k, needed_padding)
            })
            .collect();

        (widest, pad_map)
    }

    #[test]
    fn default_to_pad_map_left() {
        const DEFAULT_LEVELS: [&str; 2] = ["INFO", "ERROR"];

        let mut v = to_pad_map_left(DEFAULT_LEVELS)
            .1
            .into_iter()
            .collect::<Vec<(&str, u8)>>();
        v.sort_by(|(a, _), (b, _)| a.cmp(b));

        let expected = vec![("ERROR", 0), ("INFO", 1)];

        assert_eq!(v, expected);
    }
}
