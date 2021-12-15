use crate::data::Aggregate;
use chrono::DateTime;
use chrono_tz::Tz;
use std::collections::{BTreeMap, HashMap};

pub struct MarketDataCache {
    data: HashMap<String, BTreeMap<DateTime<Tz>, Aggregate>>,
}

impl MarketDataCache {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub(crate) fn get_data(
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

    pub(crate) fn get_last_before(&self, ticker: &str, end: DateTime<Tz>) -> Option<Aggregate> {
        let start = chrono::MIN_DATETIME.with_timezone(&end.timezone());
        self.data
            .get(ticker)?
            .range(start..=end)
            .last()
            .map(|(_, agg)| agg)
            .cloned()
    }

    pub(crate) fn store_data(&mut self, ticker: &str, data: Vec<Aggregate>) {
        self.data
            .entry(ticker.to_string())
            .and_modify(|map| {
                for datum in data.clone() {
                    map.insert(datum.datetime, datum);
                }
            })
            .or_insert_with(|| data.into_iter().map(|agg| (agg.datetime, agg)).collect());
    }
}
