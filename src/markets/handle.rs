use crate::data::Aggregate;
use crate::markets::clock::MarketState;
use chrono::DateTime;
use chrono_tz::Tz;
use rust_decimal::Decimal;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot::{self, Sender as OneshotSender};

#[derive(Clone, Debug)]
pub(crate) enum MarketRequest {
    Datetime,
    IsDone,
    IsOpen,
    Data {
        ticker: String,
        start: DateTime<Tz>,
        end: DateTime<Tz>,
    },
    GetOpen {
        ticker: String,
    },
    GetCurrent {
        ticker: String,
    },
    GetLast {
        ticker: String,
    },
    NextDatetime,
    State,
    PreviousDatetime,
    Tick,
}

#[derive(Clone, Debug)]
pub(crate) enum MarketResponse {
    Bool(bool),
    Data(Option<Vec<Aggregate>>),
    Datetime(DateTime<Tz>),
    MaybePrice(Option<Decimal>),
    State(MarketState),
    // Generic reply for when no reply is needed
    Success,
}

#[derive(Clone)]
pub struct Market {
    sender: UnboundedSender<(OneshotSender<MarketResponse>, MarketRequest)>,
}

impl Market {
    pub(crate) fn new(
        sender: UnboundedSender<(OneshotSender<MarketResponse>, MarketRequest)>,
    ) -> Self {
        Self { sender }
    }

    async fn send_request(&self, request: MarketRequest) -> MarketResponse {
        let (tx, rx) = oneshot::channel();
        self.sender.send((tx, request)).unwrap();
        rx.await.unwrap()
    }

    pub async fn datetime(&self) -> DateTime<Tz> {
        let response = self.send_request(MarketRequest::Datetime).await;
        if let MarketResponse::Datetime(dt) = response {
            dt
        } else {
            unreachable!()
        }
    }

    pub async fn previous_datetime(&self) -> DateTime<Tz> {
        let response = self.send_request(MarketRequest::PreviousDatetime).await;
        if let MarketResponse::Datetime(dt) = response {
            dt
        } else {
            unreachable!()
        }
    }

    pub async fn next_datetime(&self) -> DateTime<Tz> {
        let response = self.send_request(MarketRequest::NextDatetime).await;
        if let MarketResponse::Datetime(dt) = response {
            dt
        } else {
            unreachable!()
        }
    }

    pub async fn state(&self) -> MarketState {
        let response = self.send_request(MarketRequest::State).await;
        if let MarketResponse::State(state) = response {
            state
        } else {
            unreachable!()
        }
    }

    pub async fn is_open(&self) -> bool {
        let response = self.send_request(MarketRequest::IsOpen).await;
        if let MarketResponse::Bool(b) = response {
            b
        } else {
            unreachable!()
        }
    }

    pub async fn get_data<T: ToString>(
        &self,
        ticker: T,
        start: DateTime<Tz>,
        end: DateTime<Tz>,
    ) -> Option<Vec<Aggregate>> {
        let response = self
            .send_request(MarketRequest::Data {
                ticker: ticker.to_string(),
                start,
                end,
            })
            .await;
        if let MarketResponse::Data(data) = response {
            data
        } else {
            unreachable!()
        }
    }

    pub(crate) async fn get_current_price(&self, ticker: &str) -> Option<Decimal> {
        let response = self
            .send_request(MarketRequest::GetCurrent {
                ticker: ticker.to_string(),
            })
            .await;
        if let MarketResponse::MaybePrice(p) = response {
            p
        } else {
            unreachable!()
        }
    }

    pub async fn get_last_price(&self, ticker: &str) -> Option<Decimal> {
        let response = self
            .send_request(MarketRequest::GetLast {
                ticker: ticker.to_string(),
            })
            .await;
        if let MarketResponse::MaybePrice(p) = response {
            p
        } else {
            unreachable!()
        }
    }

    pub async fn get_open(&self, ticker: &str) -> Option<Decimal> {
        let response = self
            .send_request(MarketRequest::GetOpen {
                ticker: ticker.to_string(),
            })
            .await;
        if let MarketResponse::MaybePrice(p) = response {
            p
        } else {
            unreachable!()
        }
    }

    pub(crate) async fn is_done(&self) -> bool {
        let response = self.send_request(MarketRequest::IsDone).await;
        if let MarketResponse::Bool(b) = response {
            b
        } else {
            unreachable!()
        }
    }

    pub(crate) async fn tick(&self) {
        self.send_request(MarketRequest::Tick).await;
    }
}
