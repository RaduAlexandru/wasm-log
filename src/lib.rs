//! A simple logger for front end wasm web app.
//!
//! Please see [README](https://github.com/s1gtrap/wasm-log/blob/main/README.md) for documentation.
#![deny(missing_docs)]
use env_logger::filter::Builder;
use env_logger::filter::Filter;
use log::{Level, LevelFilter, Log, Metadata, Record};
use wasm_bindgen::prelude::*;
use web_sys::console;

/// Specify what to be logged
pub struct Config {
    // level: Level,
    // module_prefix: Option<String>,
    filter: Filter,
    message_location: MessageLocation,
}

/// Specify where the message will be logged.
pub enum MessageLocation {
    /// The message will be on the same line as other info (level, path...)
    SameLine,
    /// The message will be on its own line, a new after other info.
    NewLine,
}

impl Default for Config {
    fn default() -> Self {
        let mut builder = Builder::new();
        Self {
            // level: Level::Debug,
            filter: builder.build(),
            message_location: MessageLocation::SameLine,
        }
    }
}

impl Config {
    /// Specify the maximum level you want to log
    pub fn new(filter: Filter) -> Self {
        Self {
            // level,
            filter,
            message_location: MessageLocation::SameLine,
        }
    }

    // /// Configure the `target` of the logger. If specified, the logger
    // /// only output for `log`s in module that its path starts with
    // /// `module_prefix`. wasm-log only supports single prefix. Only
    // /// the last call to `module_prefix` has effect if you call it multiple times.
    // pub fn module_prefix(mut self, module_prefix: &str) -> Self {
    //     self.module_prefix = Some(module_prefix.to_string());
    //     self
    // }

    /// Put the message on a new line, separated from other information
    /// such as level, file path, line number.
    pub fn message_on_new_line(mut self) -> Self {
        self.message_location = MessageLocation::NewLine;
        self
    }
}

/// The log styles
struct Style {
    lvl_trace: String,
    lvl_debug: String,
    lvl_info: String,
    lvl_warn: String,
    lvl_error: String,
    tgt: String,
    args: String,
}

impl Style {
    fn new() -> Style {
        let base = String::from("color: white; padding: 0 3px; background:");
        Style {
            lvl_trace: format!("{} gray;", base),
            lvl_debug: format!("{} blue;", base),
            lvl_info: format!("{} green;", base),
            lvl_warn: format!("{} orange;", base),
            lvl_error: format!("{} darkred;", base),
            tgt: String::from("font-weight: bold; color: inherit"),
            args: String::from("background: inherit; color: inherit"),
        }
    }
}

/// The logger
struct WasmLogger {
    config: Config,
    style: Style,
}

impl Log for WasmLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.config.filter.enabled(metadata)
        // if let Some(ref prefix) = self.config.module_prefix {
        //     metadata.target().starts_with(prefix)
        // } else {
        //     true
        // }
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            let style = &self.style;
            let message_separator = match self.config.message_location {
                MessageLocation::NewLine => "\n",
                MessageLocation::SameLine => " ",
            };
            let s = format!(
                "%c{}%c {}:{}%c{}{}",
                record.level(),
                record.file().unwrap_or_else(|| record.target()),
                record
                    .line()
                    .map_or_else(|| "[Unknown]".to_string(), |line| line.to_string()),
                message_separator,
                record.args(),
            );
            let s = JsValue::from_str(&s);
            let tgt_style = JsValue::from_str(&style.tgt);
            let args_style = JsValue::from_str(&style.args);

            match record.level() {
                Level::Trace => console::debug_4(
                    &s,
                    &JsValue::from(&style.lvl_trace),
                    &tgt_style,
                    &args_style,
                ),
                Level::Debug => console::log_4(
                    &s,
                    &JsValue::from(&style.lvl_debug),
                    &tgt_style,
                    &args_style,
                ),
                Level::Info => {
                    console::info_4(&s, &JsValue::from(&style.lvl_info), &tgt_style, &args_style)
                }
                Level::Warn => {
                    console::warn_4(&s, &JsValue::from(&style.lvl_warn), &tgt_style, &args_style)
                }
                Level::Error => console::error_4(
                    &s,
                    &JsValue::from(&style.lvl_error),
                    &tgt_style,
                    &args_style,
                ),
            }
        }
    }

    fn flush(&self) {}
}

/// Initialize the logger which the given config. If failed, it will log a message to the the browser console.
///
/// ## Examples
/// ```rust
/// wasm_log::init(wasm_log::Config::new(log::Level::Debug));
/// ```
/// or
/// ```rust
/// wasm_log::init(wasm_log::Config::new(log::Level::Debug).module_prefix("some::module"));
/// ```
pub fn init(config: Config) {
    match try_init(config) {
        Ok(_) => {}
        Err(e) => console::error_1(&JsValue::from(e.to_string())),
    }
}

/// Attempt to initialize the logger with the given config.
///
/// # Errors
///
/// This function will fail if it is called more than once, or if another
/// library has already initialized a global logger.
pub fn try_init(config: Config) -> Result<(), log::SetLoggerError> {
    let max_level = config.filter.filter();
    let wl = WasmLogger {
        config,
        style: Style::new(),
    };

    match log::set_boxed_logger(Box::new(wl)) {
        Ok(_) => {
            log::set_max_level(max_level);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

#[test]
fn test_try_init() {
    assert!(try_init(Config::default()).is_ok());
    assert!(try_init(Config::default()).is_err()); // should fail on second attempt to init
}
