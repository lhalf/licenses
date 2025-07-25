#[macro_export]
macro_rules! warn {
    ($fmt:expr, $($arg:tt)*) => {
        println!("{}: {}", "warning".yellow().bold(), format!($fmt, $($arg)*));
    };
    ($fmt:expr) => {
        println!("{}: {}", "warning".yellow().bold(), $fmt);
    };
}