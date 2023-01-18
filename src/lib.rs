#![deny(warnings)]
#![deny(missing_docs)]

//! A logger configured via an environment variable which writes cancer to
//! standard error with colored output for log levels.
//!
//! ## Example
//!
//! ```
//! extern crate emoji_logger;
//! #[macro_use] extern crate log;
//!
//! fn main() {
//!     emoji_logger::init();
//!
//!     trace!("this is trace level");
//!     debug!("mom get the rubber duck");
//!     info!("heck, our disk is full of logs again...");
//!     warn!("should we worry");
//!     error!("pls help");
//! }
//! ```

extern crate ansi_term;
extern crate env_logger;
extern crate log;

use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::time::Instant;

use ansi_term::{Color, Style};
use env_logger::Builder;
use log::Level;

static MAX_MODULE_WIDTH: AtomicUsize = ATOMIC_USIZE_INIT;
static SINCE_WIDTH_INCREASE: AtomicUsize = ATOMIC_USIZE_INIT;

/// Initializes the global logger with an emoji logger.
///
/// This should be called early in the execution of a Rust program, and the
/// global logger may only be initialized once. Future initialization attempts
/// will return an error.
///
/// # Panics
///
/// This function fails to set the global logger if one has already been set.
#[inline]
pub fn init() {
    try_init().unwrap();
}

/// Initializes the global logger with an emoji logger.
///
/// This should be called early in the execution of a Rust program, and the
/// global logger may only be initialized once. Future initialization attempts
/// will return an error.
///
/// # Errors
///
/// This function fails to set the global logger if one has already been set.
pub fn try_init() -> Result<(), log::SetLoggerError> {
    try_init_custom_env("RUST_LOG")
}

/// Initialized the global logger with an emoji logger, with a custom variable
/// name.
///
/// This should be called early in the execution of a Rust program, and the
/// global logger may only be initialized once. Future initialization attempts
/// will return an error.
///
/// # Panics
///
/// This function fails to set the global logger if one has already been set.
pub fn init_custom_env(environment_variable_name: &str) {
    try_init_custom_env(environment_variable_name).unwrap();
}

/// Initialized the global logger with an emoji logger, with a custom variable
/// name.
///
/// This should be called early in the execution of a Rust program, and the
/// global logger may only be initialized once. Future initialization attempts
/// will return an error.
///
/// # Errors
///
/// This function fails to set the global logger if one has already been set.
pub fn try_init_custom_env(environment_variable_name: &str) -> Result<(), log::SetLoggerError> {
    let mut builder = formatted_builder()?;

    if let Ok(s) = ::std::env::var(environment_variable_name) {
        builder.parse(&s);
    }

    builder.try_init()
}

/// Returns a `env_logger::Builder` for further customization.
///
/// This method will return a colored and formatted) `env_logger::Builder`
/// for further customization. Refer to env_logger::Build crate documentation
/// for further details and usage.
///
/// This should be called early in the execution of a Rust program, and the
/// global logger may only be initialized once. Future initialization attempts
/// will return an error.
///
/// # Errors
///
/// This function fails to set the global logger if one has already been set.
pub fn formatted_builder() -> Result<Builder, log::SetLoggerError> {
    let mut builder = Builder::new();
    let start_time = Instant::now();

    builder.format(move |f, record| {
        use std::io::Write;
        let target = record.target();

        let time = start_time.elapsed();
        let (color, level) = match record.level() {
            Level::Trace => (Color::Purple, " 🤓 T "),
            Level::Debug => (Color::Blue, " 🤔 D "),
            Level::Info => (Color::Green, " 😋 I "),
            Level::Warn => (Color::Yellow, " 😥 W "),
            Level::Error => (Color::Red, " 😡 E "),
        };

        let mut module_iter = target.split("::");
        let krate = module_iter.next();
        let path = module_iter.last();
        let target = format!(
            " {}{} ",
            krate.unwrap_or_default(),
            path.map(|s| format!(":{}", s)).unwrap_or_default()
        );

        SINCE_WIDTH_INCREASE.fetch_add(1, Ordering::Relaxed);
        let mut max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
        if max_width <= target.len() {
            MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
            max_width = target.len();
            SINCE_WIDTH_INCREASE.store(0, Ordering::Relaxed);
        } else if SINCE_WIDTH_INCREASE.load(Ordering::Relaxed) > 5 {
            MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
            max_width = target.len();
        }

        writeln!(
            f,
            "{}{}{}> {}",
            Style::new().on(color).fg(Color::Black).paint(level),
            Style::new()
                .on(Color::White)
                .fg(Color::Black)
                .dimmed()
                .bold()
                .paint(format!(
                    " {:.2} ",
                    time.as_secs() as f64 + (f64::from(time.subsec_millis()) / 1000f64)
                )),
            Style::new()
                .bold()
                .paint(format!("{: <width$}", target, width = max_width)),
            format!("{}", record.args()).replace(
                "\n",
                &format!(
                    "\n{} ",
                    Style::new().on(color).fg(Color::White).paint(level)
                )
            )
        )
    });

    Ok(builder)
}
