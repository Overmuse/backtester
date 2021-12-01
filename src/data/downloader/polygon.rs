use crate::data::{error::Error, Aggregate, DataOptions, DataProvider, MarketData, Resolution};
use ::polygon::rest::{client, Aggregate as PolygonAggregate, GetAggregate, Timespan};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::{StreamExt, TryStreamExt};
use std::collections::{BTreeMap, HashMap};
use stream_flatten_iters::TryStreamExt as _;

pub struct PolygonDownloader;

impl From<PolygonAggregate> for Aggregate {
    fn from(p: PolygonAggregate) -> Aggregate {
        Aggregate {
            datetime: p.t,
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
    async fn download_data(&self, meta: &DataOptions) -> Result<MarketData, Error> {
        let client = client(&std::env::var("POLYGON_TOKEN").unwrap());
        let timespan = match meta.resolution {
            Resolution::Day => Timespan::Day,
            Resolution::Minute => Timespan::Minute,
        };
        let mut map: HashMap<String, BTreeMap<DateTime<Utc>, Aggregate>> = HashMap::new();
        for ticker in &meta.tickers {
            let query = GetAggregate::new(
                ticker,
                meta.start.and_hms(0, 0, 0),
                meta.end.and_hms(0, 0, 0),
            )
            .timespan(timespan)
            .limit(50000);

            let prices: BTreeMap<DateTime<Utc>, Aggregate> = client
                .send_paginated(&query)
                .map_ok(|x| x.results)
                .try_flatten_iters()
                .filter_map(|x| async { x.ok() })
                .map(|agg| (agg.t, From::from(agg)))
                .collect()
                .await;
            map.insert(ticker.to_string(), prices);
        }

        let mut data = MarketData::new(meta.tickers.clone(), map, meta.resolution);

        if meta.normalize {
            data.normalize_data()
        }

        Ok(data)
    }
}
