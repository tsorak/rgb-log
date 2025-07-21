pub mod buf;
pub mod log;
pub use log::{Log, color};

#[macro_export]
macro_rules! info {
    ($log:expr, $($arg:tt)*) => {
        $log.info(format_args!($($arg)*));
    };
    ($submodulelog:expr, $($arg:tt)*) => {
        $submodulelog.info(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! ok {
    ($log:expr, $($arg:tt)*) => {
        $log.ok(format_args!($($arg)*));
    };
    ($submodulelog:expr, $($arg:tt)*) => {
        $submodulelog.ok(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! error {
    ($log:expr, $($arg:tt)*) => {
        $log.error(format_args!($($arg)*));
    };
    ($submodulelog:expr, $($arg:tt)*) => {
        $submodulelog.error(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! debug {
    ($log:expr, $($arg:tt)*) => {
        $log.debug(format_args!($($arg)*));
    };
    ($submodulelog:expr, $($arg:tt)*) => {
        $submodulelog.debug(format_args!($($arg)*));
    };
}
