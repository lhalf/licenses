use colored::Colorize;

#[derive(PartialEq, Debug)]
pub enum LogLevel {
    Warning,
    Note,
}

pub fn log_message(level: LogLevel, message: &str) -> String {
    match level {
        LogLevel::Warning => format!("{}: {}", "warning".yellow().bold(), message),
        LogLevel::Note => format!("{}: {}", "note".green().bold(), message),
    }
}
