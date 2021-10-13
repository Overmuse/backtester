use crate::data::{Aggregate, DataOptions, DataProvider, Error, MarketData, PriceData};
use ::polygon::rest::{Aggregate as PolygonAggregate, AggregateWrapper, Client, GetAggregate};
use async_trait::async_trait;

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
        let client = Client::from_env()?;
        let queries = meta
            .tickers
            .iter()
            .map(|ticker| GetAggregate::new(ticker, meta.start, meta.end));
        let wrappers: Result<Vec<AggregateWrapper>, Error> = client
            .send_all(queries)
            .await
            .into_iter()
            .map(|x| x.map_err(From::from))
            .collect();
        let prices: PriceData = wrappers?
            .into_iter()
            .map(|w| (w.ticker, w.results.unwrap_or_default()))
            .map(|(ticker, data)| {
                (
                    ticker,
                    data.into_iter()
                        .map(|agg| (agg.t, From::from(agg)))
                        .collect(),
                )
            })
            .collect();
        Ok(MarketData { prices })
    }
}
