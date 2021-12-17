use crate::data::{provider::DataProvider, Aggregate, DataOptions, Resolution};
use crate::utils::last_day_of_month;
use chrono::prelude::*;
use chrono_tz::Tz;
use futures::{stream, StreamExt};
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap};
use tracing::error;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DownloadJob {
    ticker: String,
    start: NaiveDate,
    end: NaiveDate,
    resolution: Resolution,
}

pub struct DataManager {
    data_provider: Box<dyn DataProvider + Send + Sync>,
    download_jobs: Vec<DownloadJob>,
    data: HashMap<String, BTreeMap<DateTime<Tz>, Aggregate>>,
}

impl DataManager {
    pub fn new<D: 'static + DataProvider + Send + Sync>(
        data_provider: D,
        data_options: DataOptions,
    ) -> Self {
        let download_jobs = data_options
            .tickers
            .iter()
            .map(|ticker| DownloadJob {
                ticker: ticker.to_string(),
                start: data_options.start,
                end: data_options.end,
                resolution: data_options.resolution,
            })
            .collect();

        Self {
            data_provider: Box::new(data_provider),
            download_jobs,
            data: HashMap::new(),
        }
    }

    pub async fn download_data(&mut self) {
        let jobs = self.download_jobs.clone();
        //let progress = crate::utils::progress(jobs.len() as u64, "Downloading data");
        let data: Vec<(String, Vec<Aggregate>)> = stream::iter(jobs.into_iter())
            .map(|job| {
                let data_provider = &self.data_provider;
                //let progress = progress.clone();
                async move {
                    let data = data_provider
                        .download_data(&job.ticker, job.start, job.end, job.resolution)
                        .await;
                    if let Err(ref e) = data {
                        error!("{}", e);
                    }
                    let data = data.unwrap();
                    //progress.inc(1);
                    (job.ticker, data)
                }
            })
            .buffer_unordered(20)
            .collect()
            .await;
        //progress.finish();

        let progress = crate::utils::progress(data.len() as u64, "Storing data");
        for (ticker, datum) in data.into_iter() {
            self.data
                .entry(ticker.to_string())
                .and_modify(|map| {
                    for agg in datum.clone() {
                        map.insert(agg.datetime, agg);
                    }
                })
                .or_insert_with(|| datum.into_iter().map(|agg| (agg.datetime, agg)).collect());
            progress.inc(1);
        }
        progress.finish();
    }

    pub fn get_data(
        &self,
        ticker: &str,
        start: DateTime<Tz>,
        end: DateTime<Tz>,
    ) -> Option<Vec<Aggregate>> {
        let data: Vec<Aggregate> = self
            .data
            .get(ticker)?
            .range(start..=end)
            .map(|(_, agg)| agg)
            .cloned()
            .collect();
        if data.is_empty() {
            None
        } else {
            Some(data)
        }
    }

    pub fn get_last_before(&self, ticker: &str, datetime: DateTime<Tz>) -> Option<Aggregate> {
        let start = chrono::MIN_DATETIME.with_timezone(&datetime.timezone());
        self.data
            .get(ticker)?
            .range(start..=datetime)
            .last()
            .map(|(_, agg)| agg)
            .cloned()
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
            // Download up to 4 month's data at a time
            let mut range = Vec::new();
            let first_date = NaiveDate::from_ymd(start.year(), start.month(), 1);
            let last_date = NaiveDate::from_ymd(
                end.year(),
                end.month(),
                last_day_of_month(end.year(), end.month()),
            );

            let mut current_start = first_date;
            let mut current_end = match start.month() {
                9 | 10 | 11 | 12 => NaiveDate::from_ymd(
                    start.year() + 1,
                    (start.month() + 4) % 12,
                    last_day_of_month(start.year() + 1, (start.month() + 4) % 12),
                ),
                _ => NaiveDate::from_ymd(
                    start.year(),
                    start.month() + 4,
                    last_day_of_month(start.year(), start.month()),
                ),
            };
            range.push((current_start, current_end));
            while current_end < last_date {
                let mut yy = current_start.year();
                let mut mm = current_start.month();
                match mm {
                    9 | 10 | 11 | 12 => {
                        mm = (mm + 4) % 12;
                        yy += 1
                    }
                    _ => mm += 4,
                };
                current_start = NaiveDate::from_ymd(yy, mm, 1);
                current_end = NaiveDate::from_ymd(yy, mm, last_day_of_month(yy, mm));
                range.push((current_start, current_end));
            }
            range
        }
    }
}
