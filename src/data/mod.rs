use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::US::Eastern;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub use cache::FileCache;
mod cache;
pub mod downloader;
pub mod error;

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum Resolution {
    Minute,
    Day,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DataOptions {
    tickers: Vec<String>,
    start: NaiveDate,
    end: NaiveDate,
    resolution: Resolution,
    normalize: bool,
}

impl DataOptions {
    pub fn new(tickers: Vec<String>, start: NaiveDate, end: NaiveDate) -> Self {
        Self {
            tickers,
            start,
            end,
            resolution: Resolution::Day,
            normalize: false,
        }
    }

    pub fn resolution(mut self, resolution: Resolution) -> Self {
        self.resolution = resolution;
        self
    }

    pub fn normalize(mut self, normalize: bool) -> Self {
        self.normalize = normalize;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aggregate {
    pub datetime: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}

pub trait MarketTimeExt {
    fn is_regular_hours(&self) -> bool;
    fn is_opening(&self) -> bool;
    fn is_closing(&self) -> bool;
}

impl<T: TimeZone> MarketTimeExt for DateTime<T> {
    fn is_regular_hours(&self) -> bool {
        let zoned = self.with_timezone(&Eastern);
        (zoned.time() >= NaiveTime::from_hms(9, 30, 00))
            && (zoned.time() < NaiveTime::from_hms(16, 00, 00))
    }
    fn is_opening(&self) -> bool {
        self.with_timezone(&Eastern).time() == NaiveTime::from_hms(9, 30, 0)
    }
    fn is_closing(&self) -> bool {
        self.with_timezone(&Eastern).time() == NaiveTime::from_hms(16, 0, 0)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketData {
    timestamps: Vec<DateTime<Utc>>,
    tickers: Vec<String>,
    /// Row-major array with rows = dates and colums = tickers
    data: Vec<Vec<Option<Aggregate>>>,
    resolution: Resolution,
}

impl MarketData {
    pub fn new(
        tickers: Vec<String>,
        mut raw_data: HashMap<String, BTreeMap<DateTime<Utc>, Aggregate>>,
        resolution: Resolution,
    ) -> Self {
        let timestamps: BTreeSet<DateTime<Utc>> =
            raw_data.values().flatten().map(|x| *x.0).collect();
        let mut data: Vec<Vec<Option<Aggregate>>> = Vec::with_capacity(timestamps.len());
        timestamps
            .iter()
            .for_each(|_| data.push(Vec::with_capacity(tickers.len())));
        for ticker in tickers.iter() {
            let ticker_data = raw_data.get_mut(ticker).unwrap();
            for (i, t) in timestamps.iter().enumerate() {
                data[i].push(ticker_data.remove(t))
            }
        }
        Self {
            timestamps: timestamps.into_iter().collect(),
            tickers,
            data,
            resolution,
        }
    }
    pub fn timestamps(&self) -> &[DateTime<Utc>] {
        &self.timestamps
    }

    pub fn tickers(&self) -> &[String] {
        &self.tickers
    }

    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    pub fn get_timestamp(
        &self,
        timestamp: DateTime<Utc>,
    ) -> impl Iterator<Item = (&str, &Option<Aggregate>)> {
        let idx = self
            .timestamps
            .iter()
            .position(|d| *d == timestamp)
            .unwrap();
        self.tickers
            .iter()
            .map(String::as_str)
            .zip(self.data.get(idx).unwrap())
    }

    pub fn get_timeseries(
        &self,
        ticker: &str,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> impl Iterator<Item = (&DateTime<Utc>, &Option<Aggregate>)> {
        let ticker_idx = self.tickers.iter().position(|t| t == ticker).unwrap();
        let start_idx = start
            .map(|d| self.timestamps.iter().position(|date| *date == d))
            .flatten()
            .unwrap_or(0);
        let end_idx = end
            .map(|d| self.timestamps.iter().position(|date| *date == d))
            .flatten()
            .unwrap_or(self.timestamps.len());
        self.timestamps.iter().zip(
            self.data
                .get(start_idx..end_idx)
                .unwrap()
                .get(ticker_idx)
                .unwrap(),
        )
    }

    fn normalize_data(&mut self) {
        let idx = self.timestamps.iter().map(|t| {
            let dt_tz = t.with_timezone(&Eastern);
            (dt_tz.time() >= NaiveTime::from_hms(9, 30, 0))
                && (dt_tz.time() < NaiveTime::from_hms(16, 0, 0))
        });
        let (new_data, new_timestamps) = self
            .data
            .iter()
            .zip(self.timestamps.clone())
            .zip(idx)
            .filter_map(|((r, t), i)| if i { Some((r.clone(), t)) } else { None })
            .unzip();
        self.data = new_data;
        self.timestamps = new_timestamps;
    }
}

#[async_trait]
pub trait DataProvider {
    async fn download_data(&self, meta: &DataOptions) -> Result<MarketData, error::Error>;
}
