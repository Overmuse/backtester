use crate::data::{provider::DataProvider, Aggregate, DataOptions};
use crate::markets::clock::{Clock, MarketState};
use crate::markets::data_manager::DataManager;
use crate::markets::handle::*;
use chrono::prelude::*;
use chrono_tz::Tz;
use indicatif::ProgressBar;
use rust_decimal::Decimal;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio::sync::oneshot::Sender as OneshotSender;
use tracing::debug;

pub(crate) struct MarketActor {
    requests: UnboundedReceiver<(OneshotSender<MarketResponse>, MarketRequest)>,
    data_manager: DataManager,
    clock: Clock,
    progress: ProgressBar,
}

impl MarketActor {
    pub fn spawn<D: 'static + DataProvider + Send + Sync>(
        data_provider: D,
        data_options: DataOptions,
    ) -> Market {
        let clock = Clock::new(
            data_options.start,
            data_options.end,
            data_options.warmup,
            data_options.resolution,
        );
        let progress = crate::utils::progress(clock.simulation_periods() as u64, "Simulating");
        let data_manager = DataManager::new(data_provider, data_options);
        let (tx, rx) = unbounded_channel();
        let handle = Market::new(tx);

        let actor = Self {
            requests: rx,
            data_manager,
            clock,
            progress,
        };
        tokio::spawn(async move { actor.run_forever().await });
        handle
    }

    async fn run_forever(mut self) {
        self.data_manager.download_data().await;
        self.progress.reset();
        while let Some((tx, request)) = self.requests.recv().await {
            debug!("Received request: {:?}", request);
            let response = self.handle_message(request).await;
            tx.send(response).unwrap()
        }
        debug!("No listeners remain, disconnecting");
    }

    async fn handle_message(&mut self, request: MarketRequest) -> MarketResponse {
        match request {
            MarketRequest::Data { ticker, start, end } => {
                let data = self.get_data(&ticker, start, end).await;
                MarketResponse::Data(data)
            }
            MarketRequest::GetOpen { ticker } => {
                MarketResponse::MaybePrice(self.get_open(&ticker).await)
            }
            MarketRequest::GetCurrent { ticker } => {
                MarketResponse::MaybePrice(self.get_current_price(&ticker).await)
            }
            MarketRequest::GetLast { ticker } => {
                MarketResponse::MaybePrice(self.get_last_price(&ticker).await)
            }
            MarketRequest::Datetime => MarketResponse::Datetime(self.datetime().await),
            MarketRequest::PreviousDatetime => {
                MarketResponse::Datetime(self.previous_datetime().await)
            }
            MarketRequest::NextDatetime => MarketResponse::Datetime(self.next_datetime().await),
            MarketRequest::State => MarketResponse::State(self.state().await),
            MarketRequest::IsDone => MarketResponse::Bool(self.is_done().await),
            MarketRequest::IsOpen => MarketResponse::Bool(self.is_open().await),
            MarketRequest::Tick => {
                self.tick().await;
                MarketResponse::Success
            }
        }
    }

    async fn get_open(&self, ticker: &str) -> Option<Decimal> {
        let datetime = self.datetime().await;
        let data = self.get_data(ticker, datetime, datetime).await;
        data.map(|x| x.first().unwrap().open)
    }

    async fn get_current_price(&self, ticker: &str) -> Option<Decimal> {
        let datetime = self.datetime().await;
        self.data_manager
            .get_last_before(ticker, datetime)
            .map(|x| x.close)
    }

    pub async fn get_last_price(&self, ticker: &str) -> Option<Decimal> {
        let datetime = self.previous_datetime().await;
        let data = self.get_data(ticker, datetime, datetime).await;
        data.map(|x| x.last().unwrap().close)
    }

    async fn get_data(
        &self,
        ticker: &str,
        start: DateTime<Tz>,
        end: DateTime<Tz>,
    ) -> Option<Vec<Aggregate>> {
        self.data_manager.get_data(ticker, start, end).await
    }

    async fn datetime(&self) -> DateTime<Tz> {
        self.clock.datetime()
    }

    async fn state(&self) -> MarketState {
        self.clock.state()
    }

    async fn is_done(&self) -> bool {
        if self.clock.is_done() {
            self.progress.finish();
            true
        } else {
            false
        }
    }

    async fn is_open(&self) -> bool {
        self.clock.is_open()
    }

    async fn previous_datetime(&self) -> DateTime<Tz> {
        self.clock.previous_datetime()
    }

    async fn next_datetime(&self) -> DateTime<Tz> {
        self.clock.next_datetime()
    }

    async fn tick(&mut self) {
        self.clock.tick();
        self.progress.inc(1);
    }
}
