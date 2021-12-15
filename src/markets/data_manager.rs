use crate::data::{provider::DataProvider, Aggregate, DataOptions, Resolution};
use crate::markets::cache::MarketDataCache;
use crate::utils::last_day_of_month;
use chrono::prelude::*;
use chrono_tz::Tz;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use priority_queue::PriorityQueue;
use std::collections::HashMap;
use tracing::{debug, trace};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DownloadJob {
    ticker: String,
    start: NaiveDate,
    end: NaiveDate,
    resolution: Resolution,
}

pub struct DataManager {
    cache: MarketDataCache,
    data_provider: Box<dyn DataProvider + Send + Sync>,
    data_options: DataOptions,
    download_jobs: PriorityQueue<DownloadJob, usize>,
    date_ranges: HashMap<String, (NaiveDate, NaiveDate)>,
    progress: ProgressBar,
}

impl DataManager {
    pub fn new<D: 'static + DataProvider + Send + Sync>(
        data_provider: D,
        data_options: DataOptions,
    ) -> Self {
        let mut download_jobs = PriorityQueue::new();
        let mut jobs = build_date_range(
            data_options.start,
            data_options.end,
            data_options.resolution,
        );
        // Reverse to give earlier jobs higher priority
        jobs.reverse();
        let jobs = jobs
            .into_iter()
            // Enumerate to be given a priority
            .enumerate()
            // Only do cartesian product after enumerating so each ticker has same priority
            .cartesian_product(data_options.tickers.iter())
            .map(|((priority, (start, end)), ticker)| {
                (
                    priority,
                    DownloadJob {
                        ticker: ticker.to_string(),
                        start,
                        end,
                        resolution: data_options.resolution,
                    },
                )
            });
        for (priority, job) in jobs {
            download_jobs.push(job, priority);
        }
        let cache = MarketDataCache::new();
        let date_ranges = HashMap::new();
        let progress = ProgressBar::new(download_jobs.len() as u64)
            .with_message("Downloading data")
            .with_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                    .progress_chars("##-"),
            );

        Self {
            cache,
            data_provider: Box::new(data_provider),
            data_options,
            download_jobs,
            date_ranges,
            progress,
        }
    }

    pub fn is_done_downloading(&self) -> bool {
        self.download_jobs.is_empty()
    }

    pub fn has_data(&self, ticker: &str, start: DateTime<Tz>, end: DateTime<Tz>) -> bool {
        if let Some((s, e)) = self.date_ranges.get(ticker) {
            s <= &start.naive_local().date() && e >= &end.naive_local().date()
        } else {
            false
        }
    }

    pub async fn get_data(
        &mut self,
        ticker: &str,
        start: DateTime<Tz>,
        end: DateTime<Tz>,
    ) -> Option<Vec<Aggregate>> {
        if !self.has_data(ticker, start, end) {
            // We push the job to the queue rather than processing directly in order to
            // override any existing priority
            debug!("Data not yet downloaded. {}, {}-{}", ticker, start, end);
            let date_ranges = build_date_range(
                self.data_options.start,
                self.data_options.end,
                self.data_options.resolution,
            );

            for (start, end) in date_ranges {
                let job = DownloadJob {
                    ticker: ticker.to_string(),
                    start,
                    end,
                    resolution: self.data_options.resolution,
                };
                self.download_jobs.push(job, usize::MAX);
                self.process_download_job().await;
            }
        };
        // Now the data should exist
        self.cache.get_data(ticker, start, end)
    }

    pub async fn process_job(&mut self, job: DownloadJob) {
        let data = self
            .data_provider
            .download_data(&job.ticker, job.start, job.end, job.resolution)
            .await
            .unwrap();
        self.cache.store_data(&job.ticker, data);
        self.date_ranges
            .entry(job.ticker.clone())
            .and_modify(|(s, e)| {
                if job.start < *s {
                    *s = job.start
                }
                if job.end > *e {
                    *e = job.end
                }
            })
            .or_insert((job.start, job.end));
        self.progress.inc(1);
    }

    pub async fn process_download_job(&mut self) {
        if let Some((job, _)) = self.download_jobs.pop() {
            trace!("Download job: {:?}", job);
            self.process_job(job).await;
        }
    }

    pub fn get_last_before(&self, ticker: &str, datetime: DateTime<Tz>) -> Option<Aggregate> {
        self.cache.get_last_before(ticker, datetime)
    }
}

fn build_date_range(
    start: NaiveDate,
    end: NaiveDate,
    resolution: Resolution,
) -> Vec<(NaiveDate, NaiveDate)> {
    match resolution {
        Resolution::Day => {
            // Download up to a year's data at a time
            let mut range = Vec::new();
            let first_date = NaiveDate::from_ymd(start.year(), 1, 1);
            let last_date = NaiveDate::from_ymd(end.year(), 12, 31);
            let mut current_start = first_date;
            let mut current_end = NaiveDate::from_ymd(start.year(), 12, 31);
            range.push((current_start, current_end));
            while current_end < last_date {
                current_start = NaiveDate::from_ymd(current_start.year() + 1, 1, 1);
                current_end = NaiveDate::from_ymd(current_end.year() + 1, 12, 31);
                range.push((current_start, current_end));
            }
            range
        }
        Resolution::Minute => {
            // Download up to a month's data at a time
            let mut range = Vec::new();
            let first_date = NaiveDate::from_ymd(start.year(), start.month(), 1);
            let last_date = NaiveDate::from_ymd(
                end.year(),
                end.month(),
                last_day_of_month(end.year(), end.month()),
            );

            let mut current_start = first_date;
            let mut current_end = NaiveDate::from_ymd(
                start.year(),
                start.month(),
                last_day_of_month(start.year(), start.month()),
            );
            range.push((current_start, current_end));
            while current_end < last_date {
                let mut yy = current_start.year();
                let mut mm = current_start.month();
                if mm == 12 {
                    mm = 1;
                    yy += 1
                } else {
                    mm += 1
                }
                current_start = NaiveDate::from_ymd(yy, mm, 1);
                current_end = NaiveDate::from_ymd(yy, mm, last_day_of_month(yy, mm));
                range.push((current_start, current_end));
            }
            range
        }
    }
}
