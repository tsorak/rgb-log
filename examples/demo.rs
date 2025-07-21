use std::{error::Error, fmt::Display};

use rgb_log::{Log, debug, error};

const SUBMODULE_NAMES: [&str; 2] = ["very_long_subject", "main"];

#[derive(Debug)]
struct Displayable(&'static str);

#[allow(unused)]
#[derive(Debug)]
struct Debugable(&'static str);

fn main() {
    let log = Log::builder().with_submodule_names(SUBMODULE_NAMES).build();

    log.info("42");

    if let Err(err) = a_submodule::erroring_fn(log.submodule("very_long_subject")) {
        error!(log, "submodule error: {}", err);
    }

    let d = ghost_submodule::op(log.submodule("ghost_mod"));
    // debug!(log, "ghost data: {d}"); rust-analyzer error: d does not implement Display
    debug!(log, "ghost data: {d:#?}");
}

mod a_submodule {
    use rgb_log::log::SubmoduleLog;

    pub fn erroring_fn(log: SubmoduleLog) -> Result<(), impl std::error::Error> {
        log.debug("some runtime data");

        Err(super::Displayable("I implement Error"))
    }
}

mod ghost_submodule {
    use rgb_log::log::SubmoduleLog;

    pub fn op(log: SubmoduleLog) -> super::Debugable {
        log.ok("I am not specified in SUBMODULE_NAMES");
        super::Debugable("I am an awkward structure")
    }
}

impl Display for Displayable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for Displayable {}
