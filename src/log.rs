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
        level: LogLevel,
        #[cfg_attr(test, autospy(into = "String", with = "strip_ansi_escapes::strip_str"))] message: &str,
    );
}

impl Log for Logger {
    fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Warning => println!("{}: {}", "warning".yellow().bold(), message),
            LogLevel::Note => println!("{}: {}", "note".green().bold(), message),
        }
    }
}
