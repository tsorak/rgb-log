pub enum ProgramName {
    CrateName,
    Custom(&'static str),
    Disable,
}

impl ProgramName {
    pub fn try_to_string(&self) -> Option<String> {
        match self {
            Self::CrateName => std::env::var("CARGO_PKG_NAME").ok(),
            Self::Custom(s) => Some(s.to_string()),
            Self::Disable => None,
        }
    }
}

impl From<&'static str> for ProgramName {
    fn from(value: &'static str) -> Self {
        match value {
            "CRATE_NAME" | "PKG_NAME" | "CARGO_PKG_NAME" => ProgramName::CrateName,
            s => ProgramName::Custom(s),
        }
    }
}

impl From<Option<()>> for ProgramName {
    fn from(_value: Option<()>) -> Self {
        Self::Disable
    }
}
