use crate::data::Aggregate;
use crate::{Options, Resolution};
use chrono::prelude::*;
use chrono_tz::{Tz, US::Eastern};
use futures::{stream, StreamExt, TryStreamExt};
use polygon::rest::{client, GetAggregate, Timespan};
use std::collections::{BTreeMap, HashMap};
use stream_flatten_iters::TryStreamExt as _;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DownloadJob {
    ticker: String,
    start: NaiveDate,
    end: NaiveDate,
    resolution: Resolution,
}

pub struct DataManager {
    download_jobs: Vec<GetAggregate>,
    data: HashMap<String, BTreeMap<DateTime<Tz>, Aggregate>>,
}

impl DataManager {
    pub fn new(data_options: Options) -> Self {
        let download_jobs = data_options
            .tickers
            .iter()
            .map(|ticker| {
                let timespan = match data_options.resolution {
                    Resolution::Day => Timespan::Day,
                    Resolution::Minute => Timespan::Minute,
                };
                let start = data_options.start.and_hms(0, 0, 0);
                let end = data_options.end.and_hms(0, 0, 0);
                GetAggregate::new(ticker, start, end)
                    .timespan(timespan)
                    .limit(50000)
            })
            .collect();

        Self {
            download_jobs,
            data: HashMap::new(),
        }
    }

    pub async fn download_data(&mut self) {
        let jobs = self.download_jobs.clone();
        let client = client(&std::env::var("POLYGON_TOKEN").expect(
            "The Polygon data provider requires the POLYGON_TOKEN environment variable to be set",
        ))
        .show_progress();
        let data = stream::select_all(client.send_all_paginated(jobs.iter()).map(|stream| {
            stream
                .map_ok(|wrapper| {
                    let ticker = wrapper.ticker.clone();
                    wrapper
                        .results
                        .into_iter()
                        .map(move |r| (ticker.clone(), r))
                })
                .try_flatten_iters()
        }))
        .filter_map(|x| async move { x.ok() })
        .map(|x| async { x })
        .buffer_unordered(500);

        let mut data = Box::pin(data);
        while let Some((ticker, agg)) = data.next().await {
            self.data
                .entry(ticker.to_string())
                .and_modify(|map| {
                    let agg = agg.clone();
                    map.insert(agg.t.with_timezone(&Eastern), From::from(agg));
                })
                .or_insert_with(|| {
                    let mut map = BTreeMap::new();
                    map.insert(agg.t.with_timezone(&Eastern), From::from(agg));
                    map
                });
        }
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
