pub mod nyse_calendar;
use indicatif::{ProgressBar, ProgressStyle};
pub mod serde_tz;
use chrono::{Datelike, NaiveDate};

pub fn last_day_of_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(year, month + 1, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd(year + 1, 1, 1))
        .pred()
        .day()
}

pub fn progress(len: u64, message: &'static str) -> ProgressBar {
    ProgressBar::new(len).with_message(message).with_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} [{eta}] {msg}")
            .progress_chars("##-"),
    )
}
