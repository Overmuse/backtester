use crate::data::{provider::DataProvider, Aggregate, DataOptions};
use crate::markets::clock::{Clock, MarketState};
use crate::markets::data_manager::DataManager;
use crate::markets::handle::*;
use chrono::prelude::*;
use chrono_tz::Tz;
use indicatif::{ProgressBar, ProgressStyle};
use rust_decimal::Decimal;
use tokio::sync::mpsc::{error::TryRecvError, unbounded_channel, UnboundedReceiver};
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
        let progress = ProgressBar::new(clock.simulation_periods() as u64)
            .with_message("Simulating")
            .with_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                    .progress_chars("##-"),
            );
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
        loop {
            match self.requests.try_recv() {
                Ok((tx, request)) => {
                    debug!("Received request: {:?}", request);
                    let response = self.handle_message(request).await;
                    tx.send(response).unwrap()
                }
                Err(TryRecvError::Empty) => {
                    // No current message, let's download some data!
                    if !self.data_manager.is_done_downloading() {
                        trace!("No outstanding requests, downloading data");
                        self.data_manager.process_download_job().await;
                        trace!("Finished downloading data");
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    debug!("No listeners remain, disconnecting");
                    return;
                }
            }
            tokio::task::yield_now().await
        }
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

    async fn get_open(&mut self, ticker: &str) -> Option<Decimal> {
        let datetime = self.datetime();
        let data = self.get_data(ticker, datetime, datetime).await;
        data.map(|x| x.first().unwrap().open)
    }

    async fn get_current_price(&mut self, ticker: &str) -> Option<Decimal> {
        let datetime = self.datetime();
        self.data_manager
            .get_last_before(ticker, datetime)
            .map(|x| x.close)
    }

    pub async fn get_last_price(&mut self, ticker: &str) -> Option<Decimal> {
        let datetime = self.previous_datetime();
        let data = self.get_data(ticker, datetime, datetime).await;
        data.map(|x| x.last().unwrap().close)
    }

    async fn get_data(
        &mut self,
        ticker: &str,
        start: DateTime<Tz>,
        end: DateTime<Tz>,
    ) -> Option<Vec<Aggregate>> {
        self.data_manager.get_data(ticker, start, end).await
    }

    fn datetime(&self) -> DateTime<Tz> {
        self.clock.datetime()
    }

    fn state(&self) -> MarketState {
        self.clock.state()
    }

    fn is_done(&self) -> bool {
        if self.clock.is_done() {
            self.progress.finish();
            true
        } else {
            false
        }
    }

    fn is_open(&self) -> bool {
        self.clock.is_open()
    }

    fn previous_datetime(&self) -> DateTime<Tz> {
        self.clock.previous_datetime()
    }

    fn next_datetime(&self) -> DateTime<Tz> {
        self.clock.next_datetime()
    }

    fn tick(&mut self) {
        self.clock.tick();
        self.progress.inc(1);
    }
}
