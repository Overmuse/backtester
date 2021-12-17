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
use tracing::{debug, trace};

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
            trace!("Received request: {:?}", request);
            let response = self.handle_message(request);
            tx.send(response).unwrap()
        }
        debug!("No listeners remain, disconnecting");
    }

    fn handle_message(&mut self, request: MarketRequest) -> MarketResponse {
        match request {
            MarketRequest::Data { ticker, start, end } => {
                let data = self.get_data(&ticker, start, end);
                MarketResponse::Data(data)
            }
            MarketRequest::GetOpen { ticker } => MarketResponse::MaybePrice(self.get_open(&ticker)),
            MarketRequest::GetCurrent { ticker } => {
                MarketResponse::MaybePrice(self.get_current_price(&ticker))
            }
            MarketRequest::GetLast { ticker } => {
                MarketResponse::MaybePrice(self.get_last_price(&ticker))
            }
            MarketRequest::Datetime => MarketResponse::Datetime(self.datetime()),
            MarketRequest::PreviousDatetime => MarketResponse::Datetime(self.previous_datetime()),
            MarketRequest::NextDatetime => MarketResponse::Datetime(self.next_datetime()),
            MarketRequest::State => MarketResponse::State(self.state()),
            MarketRequest::IsDone => MarketResponse::Bool(self.is_done()),
            MarketRequest::IsOpen => MarketResponse::Bool(self.is_open()),
            MarketRequest::Tick => {
                self.tick();
                MarketResponse::Success
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn get_open(&self, ticker: &str) -> Option<Decimal> {
        trace!(ticker, "Get open");
        let datetime = self.datetime();
        let data = self.get_data(ticker, datetime, datetime);
        data.map(|x| x.first().unwrap().open)
    }

    #[tracing::instrument(skip(self))]
    fn get_current_price(&self, ticker: &str) -> Option<Decimal> {
        trace!(ticker, "Get current price");
        let datetime = self.datetime();
        self.data_manager
            .get_last_before(ticker, datetime)
            .map(|x| x.close)
    }

    #[tracing::instrument(skip(self))]
    pub fn get_last_price(&self, ticker: &str) -> Option<Decimal> {
        trace!(ticker, "Get last price");
        let datetime = self.previous_datetime();
        let data = self.get_data(ticker, datetime, datetime);
        data.map(|x| x.last().unwrap().close)
    }

    #[tracing::instrument(skip(self))]
    fn get_data(
        &self,
        ticker: &str,
        start: DateTime<Tz>,
        end: DateTime<Tz>,
    ) -> Option<Vec<Aggregate>> {
        trace!(ticker, %start, %end, "Get data");
        self.data_manager.get_data(ticker, start, end)
    }

    #[tracing::instrument(skip(self))]
    fn datetime(&self) -> DateTime<Tz> {
        self.clock.datetime()
    }

    #[tracing::instrument(skip(self))]
    fn state(&self) -> MarketState {
        self.clock.state()
    }

    #[tracing::instrument(skip(self))]
    fn is_done(&self) -> bool {
        if self.clock.is_done() {
            self.progress.finish();
            true
        } else {
            false
        }
    }

    #[tracing::instrument(skip(self))]
    fn is_open(&self) -> bool {
        self.clock.is_open()
    }

    #[tracing::instrument(skip(self))]
    fn previous_datetime(&self) -> DateTime<Tz> {
        self.clock.previous_datetime()
    }

    #[tracing::instrument(skip(self))]
    fn next_datetime(&self) -> DateTime<Tz> {
        self.clock.next_datetime()
    }

    #[tracing::instrument(skip(self))]
    fn tick(&mut self) {
        self.clock.tick();
        self.progress.inc(1);
    }
}
