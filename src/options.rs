use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Resolution {
    Minute,
    Day,
}

#[serde_as]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Options {
    pub tickers: Vec<String>,
    pub start: NaiveDate,
    pub end: NaiveDate,
    #[serde_as(as = "DurationSeconds<i64>")]
    pub warmup: Duration,
    pub resolution: Resolution,
    pub normalize: bool,
    pub outdir: Option<String>,
}

impl Options {
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
