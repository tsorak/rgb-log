use std::sync::Arc;

use crossterm::style::StyledContent;

use crate::{
    buf::LogBuffer,
    color::GetColor,
    log::{DEFAULT_LEVELS, PadLeft},
};

#[derive(Default)]
pub struct Builder {
    buffer: Option<Box<dyn LogBuffer>>,
    submodule_names: Option<Box<dyn Iterator<Item = &'static str>>>,
    levels: Option<Box<dyn Iterator<Item = Box<dyn GetColor>>>>,
}

impl Builder {
    pub fn with_buffer(mut self, bfr: impl LogBuffer) -> Self {
        let _ = self.buffer.insert(Box::new(bfr));
        self
    }

    pub fn with_submodule_names(
        mut self,
        iter: impl IntoIterator<Item = &'static str> + 'static,
    ) -> Self {
        let _ = self.submodule_names.insert(Box::new(iter.into_iter()));
        self
    }

    pub fn with_levels(
        mut self,
        iter: impl IntoIterator<Item = Box<dyn GetColor>> + 'static,
    ) -> Self {
        let _ = self.levels.insert(Box::new(iter.into_iter()));
        self
    }

    pub fn build(self) -> Arc<super::Log> {
        let bfr = self.buffer.unwrap_or(Box::new(None));
        let submodule_names = self.submodule_names.unwrap_or(Box::new([].into_iter()));
        let levels: Vec<(&'static str, StyledContent<&'static str>)> =
            if let Some(iter) = self.levels {
                iter.map(|v| (v.get_inner_str(), v.get_colored_str()))
                    .collect()
            } else {
                DEFAULT_LEVELS
                    .into_iter()
                    .map(|v| (v.get_inner_str(), v.get_colored_str()))
                    .collect()
            };

        super::Log::new_raw(bfr, submodule_names, levels)
    }
}

impl super::Log {
    pub fn new_raw<I1, I2>(buffer: Box<dyn LogBuffer>, submodule_names: I1, levels: I2) -> Arc<Self>
    where
        I1: IntoIterator<Item = &'static str>,
        I2: IntoIterator<Item = (&'static str, StyledContent<&'static str>)> + Clone,
    {
        let level_names = levels.clone().into_iter().map(|(s, _)| s);

        Self {
            buf: buffer,
            submodule_pad: PadLeft::new(submodule_names),
            level_pad: PadLeft::new(level_names),
            level_color: levels.into_iter().collect(),
        }
        .into()
    }

    pub fn builder() -> Builder {
        Builder::default()
    }
}
