use colored::Colorize;

pub struct Logger {}

pub enum LogLevel {
    Warning,
    Note,
}

#[cfg_attr(test, autospy::autospy)]
pub trait Log {
    fn log(&self, level: LogLevel, message: &str);
}

impl Log for Logger {
    fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Warning => println!("{}: {}", "warning".yellow().bold(), message),
            LogLevel::Note => println!("{}: {}", "note".green().bold(), message),
        }
    }
}
