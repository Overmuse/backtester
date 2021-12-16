use crate::data::{error::Error, provider::DataProvider, Aggregate, Resolution};
use ::polygon::rest::{client, Aggregate as PolygonAggregate, GetAggregate, Timespan};
use async_trait::async_trait;
use chrono::NaiveDate;
use chrono_tz::US::Eastern;
use futures::{StreamExt, TryStreamExt};
use stream_flatten_iters::TryStreamExt as _;

#[derive(Clone)]
pub struct PolygonDownloader;

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

#[async_trait]
impl DataProvider for PolygonDownloader {
    async fn download_data(
        &self,
        ticker: &str,
        start: NaiveDate,
        end: NaiveDate,
        resolution: Resolution,
    ) -> Result<Vec<Aggregate>, Error> {
        let client = client(&std::env::var("POLYGON_TOKEN").expect(
            "The Polygon data provider requires the POLYGON_TOKEN environment variable to be set",
        ));
        let timespan = match resolution {
            Resolution::Day => Timespan::Day,
            Resolution::Minute => Timespan::Minute,
        };
        let query = GetAggregate::new(ticker, start.and_hms(0, 0, 0), end.and_hms(0, 0, 0))
            .timespan(timespan)
            .limit(50000);

        let data = client
            .send_paginated(&query)
            .map_ok(|x| x.results)
            .try_flatten_iters()
            .filter_map(|x| async { x.ok() })
            .map(From::from)
            .collect()
            .await;
        Ok(data)
    }
}
