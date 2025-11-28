use colored::Colorize;
pub fn warning(message: &str) -> String {
    format!("{}: {}", "warning".yellow().bold(), message)
}

pub fn progress_bar() -> indicatif::ProgressBar {
    indicatif::ProgressBar::new(0).with_style(
        indicatif::ProgressStyle::with_template(
            "{spinner} checking licenses...\n{wide_bar} {pos}/{len}",
        )
        .expect("invalid progress bar style"),
    )
}

#[cfg_attr(test, autospy::autospy)]
pub trait ProgressBar {
    fn set_len(&self, len: u64);
    fn increment(&self);
}

impl ProgressBar for indicatif::ProgressBar {
    fn set_len(&self, len: u64) {
        self.set_length(len);
    }
    fn increment(&self) {
        self.inc(1);
    }
}
