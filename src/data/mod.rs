use crate::utils::serde_tz;
use chrono::{DateTime, Duration, NaiveDate, NaiveTime, TimeZone};
use chrono_tz::{Tz, US::Eastern};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};

//pub use cache::FileCache;
//mod cache;
pub mod error;
pub mod provider;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Resolution {
    Minute,
    Day,
}

#[serde_as]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DataOptions {
    pub tickers: Vec<String>,
    pub start: NaiveDate,
    pub end: NaiveDate,
    #[serde_as(as = "DurationSeconds<i64>")]
    pub warmup: Duration,
    pub resolution: Resolution,
    pub normalize: bool,
    pub outdir: Option<String>,
}

impl DataOptions {
    pub fn new(tickers: Vec<String>, start: NaiveDate, end: NaiveDate) -> Self {
        Self {
            tickers,
            start,
            end,
            warmup: Duration::zero(),
            resolution: Resolution::Day,
            normalize: false,
            outdir: None,
        }
    }

    pub fn set_resolution(mut self, resolution: Resolution) -> Self {
        self.resolution = resolution;
        self
    }

    pub fn set_warmup(mut self, warmup: Duration) -> Self {
        self.warmup = warmup;
        self
    }

    pub fn set_normalize(mut self, normalize: bool) -> Self {
        self.normalize = normalize;
        self
    }

    pub fn set_outdir<T: ToString>(mut self, outdir: T) -> Self {
        self.outdir = Some(outdir.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aggregate {
    #[serde(with = "serde_tz")]
    pub datetime: DateTime<Tz>,
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
