use crate::utils::serde_tz;
use chrono::{DateTime, NaiveTime, TimeZone};
use chrono_tz::{Tz, US::Eastern};
use polygon::rest::Aggregate as PolygonAggregate;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

pub mod error;

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

impl From<PolygonAggregate> for Aggregate {
    fn from(p: PolygonAggregate) -> Aggregate {
        Aggregate {
            datetime: p.t.with_timezone(&Eastern),
            open: p.o,
            high: p.h,
            low: p.l,
            close: p.c,
            volume: p.v,
        }
    }
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
