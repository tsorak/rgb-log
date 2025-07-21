use std::{collections::HashMap, sync::Arc};

use crossterm::style::StyledContent;

mod buf;
mod builder;
pub mod color;
pub mod padding;
pub mod program_name;

pub use buf::LogBuffer;
use color::Color;
use padding::PadLeft;
use program_name::ProgramName;

pub use builder::Builder as LogBuilder;

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
        I1: IntoIterator<Item = &'static str>,
        I2: IntoIterator<Item = (&'static str, StyledContent<&'static str>)> + Clone,
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
