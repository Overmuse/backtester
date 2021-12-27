pub mod nyse_calendar;
use indicatif::{ProgressBar, ProgressStyle};
pub mod serde_tz;

pub fn progress(len: u64, message: &'static str) -> ProgressBar {
    ProgressBar::new(len).with_message(message).with_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar} {pos:>7}/{len:7} [{eta}] {msg}"),
    )
}
