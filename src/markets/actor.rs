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
use tokio::sync::RwLock;
use tracing::debug;

pub(crate) struct MarketActor {
    requests: UnboundedReceiver<(OneshotSender<MarketResponse>, MarketRequest)>,
    data_manager: RwLock<DataManager>,
    clock: RwLock<Clock>,
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
        let data_manager = RwLock::new(DataManager::new(data_provider, data_options));
        let (tx, rx) = unbounded_channel();
        let handle = Market::new(tx);

        let actor = Self {
            requests: rx,
            data_manager,
            clock: RwLock::new(clock),
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
                    // if !self.data_manager.read().await.is_done_downloading() {
                    //     trace!("No outstanding requests, downloading data");
                    //     self.data_manager.write().await.process_download_job().await;
                    //     trace!("Finished downloading data");
                    // }
                }
                Err(TryRecvError::Disconnected) => {
                    debug!("No listeners remain, disconnecting");
                    return;
                }
            }
            tokio::task::yield_now().await
        }
    }

    async fn handle_message(&self, request: MarketRequest) -> MarketResponse {
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
            .write()
            .await
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
        self.data_manager
            .write()
            .await
            .get_data(ticker, start, end)
            .await
    }

    async fn datetime(&self) -> DateTime<Tz> {
        self.clock.read().await.datetime()
    }

    async fn state(&self) -> MarketState {
        self.clock.read().await.state()
    }

    async fn is_done(&self) -> bool {
        if self.clock.read().await.is_done() {
            self.progress.finish();
            true
        } else {
            false
        }
    }

    async fn is_open(&self) -> bool {
        self.clock.read().await.is_open()
    }

    async fn previous_datetime(&self) -> DateTime<Tz> {
        self.clock.read().await.previous_datetime()
    }

    async fn next_datetime(&self) -> DateTime<Tz> {
        self.clock.read().await.next_datetime()
    }

    async fn tick(&self) {
        self.clock.write().await.tick();
        self.progress.inc(1);
    }
}
