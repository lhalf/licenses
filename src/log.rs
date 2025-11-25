use colored::Colorize;

pub struct Logger {}

#[derive(PartialEq, Debug)]
pub enum LogLevel {
    Warning,
    Note,
}

#[cfg_attr(test, autospy::autospy)]
pub trait Log {
    fn log(
        &self,
        #[cfg_attr(test, autospy(ignore))] level: LogLevel,
        #[cfg_attr(test, autospy(into = "String", with = "strip_ansi_escapes::strip_str"))] message: &str,
    );
}

impl Log for Logger {
    fn log(&self, level: LogLevel, message: &str) {
        println!("{}", log_message(level, message));
    }
}

pub fn log_message(level: LogLevel, message: &str) -> String {
    match level {
        LogLevel::Warning => format!("{}: {}", "warning".yellow().bold(), message),
        LogLevel::Note => format!("{}: {}", "note".green().bold(), message),
    }
}
