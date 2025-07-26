#[macro_export]
macro_rules! warn {
    ($fmt:expr, $($arg:tt)*) => {
        println!("{}: {}", "warning".yellow().bold(), format!($fmt, $($arg)*));
    };
    ($fmt:expr) => {
        println!("{}: {}", "warning".yellow().bold(), $fmt);
    };
}

#[macro_export]
macro_rules! note {
    ($fmt:expr, $($arg:tt)*) => {
        println!("{}: {}", "note".green().bold(), format!($fmt, $($arg)*));
    };
    ($fmt:expr) => {
        println!("{}: {}", "note".green().bold(), $fmt);
    };
}
