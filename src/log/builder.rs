use std::sync::Arc;

use crossterm::style::StyledContent;

use crate::log::{DEFAULT_LEVELS, LogBuffer, PadLeft, color::GetColor, program_name::ProgramName};

#[derive(Default)]
pub struct Builder {
    buffer: Option<Box<dyn LogBuffer>>,
    program_name: Option<ProgramName>,
    submodule_names: Option<PadLeft<'static>>,
    levels: Option<Vec<(&'static str, StyledContent<&'static str>)>>,
}

impl Builder {
    pub fn with_buffer(mut self, bfr: impl LogBuffer) -> Self {
        let _ = self.buffer.insert(Box::new(bfr));
        self
    }

    pub fn with_program_name(mut self, v: impl Into<ProgramName>) -> Self {
        let _ = self.program_name.insert(v.into());
        self
    }

    pub fn with_submodule_names<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = &'static str>,
    {
        let _ = self.submodule_names.insert(PadLeft::new(iter));
        self
    }

    pub fn with_levels<I, T>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: GetColor,
    {
        let _ = self.levels.insert(
            iter.into_iter()
                .map(|v| (v.get_inner_str(), v.get_colored_str()))
                .collect(),
        );
        self
    }

    pub fn build(self) -> Arc<super::Log> {
        let buf = self.buffer.unwrap_or(Box::new(None));
        let program_name = self
            .program_name
            .unwrap_or(ProgramName::CrateName)
            .try_to_string();
        let submodule_names = self.submodule_names.unwrap_or_default();
        let levels: Vec<(&'static str, StyledContent<&'static str>)> =
            if let Some(vec) = self.levels {
                vec
            } else {
                DEFAULT_LEVELS
                    .into_iter()
                    .map(|v| (v.get_inner_str(), v.get_colored_str()))
                    .collect()
            };

        super::Log::new_raw(buf, program_name, submodule_names, levels)
    }
}

impl super::Log {
    fn new_raw<I>(
        buf: Box<dyn LogBuffer>,
        program_name: Option<String>,
        submodule_pad: PadLeft<'static>,
        levels: I,
    ) -> Arc<Self>
    where
        I: IntoIterator<Item = (&'static str, StyledContent<&'static str>)> + Clone,
    {
        let level_names = levels.clone().into_iter().map(|(s, _)| s);

        Self {
            buf,
            program_name,
            submodule_pad,
            level_pad: PadLeft::new(level_names),
            level_color: levels.into_iter().collect(),
        }
        .into()
    }

    pub fn builder() -> Builder {
        Builder::default()
    }
}
