use crate::brokerage::actor::Event;
use crate::brokerage::order::Order;
use crate::brokerage::position::Position;
use rust_decimal::Decimal;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::{self, Sender as OneshotSender};

#[derive(Clone, Debug)]
pub(crate) enum BrokerageRequest {
    GetPositions,
    GetEquity,
    ClosePositions,
    SendOrder(Order),
    ReconcileOrders,
    ExpireOrders,
    Subscribe,
}

#[derive(Debug)]
pub(crate) enum BrokerageResponse {
    Positions(Vec<Position>),
    Decimal(Decimal),
    EventListener(UnboundedReceiver<Event>),
    // Generic reply for when no reply is needed
    Success,
}

#[derive(Clone)]
pub struct Brokerage {
    sender: UnboundedSender<(OneshotSender<BrokerageResponse>, BrokerageRequest)>,
}

impl Brokerage {
    pub(crate) fn new(
        sender: UnboundedSender<(OneshotSender<BrokerageResponse>, BrokerageRequest)>,
    ) -> Self {
        Self { sender }
    }

    async fn send_request(&self, request: BrokerageRequest) -> BrokerageResponse {
        let (tx, rx) = oneshot::channel();
        self.sender.send((tx, request)).unwrap();
        rx.await.unwrap()
    }

    pub async fn get_positions(&self) -> Vec<Position> {
        let response = self.send_request(BrokerageRequest::GetPositions).await;
        if let BrokerageResponse::Positions(positions) = response {
            positions
        } else {
            unreachable!()
        }
    }

    pub async fn get_equity(&self) -> Decimal {
        let response = self.send_request(BrokerageRequest::GetEquity).await;
        if let BrokerageResponse::Decimal(equity) = response {
            equity
        } else {
            unreachable!()
        }
    }

    pub async fn close_positions(&self) {
        self.send_request(BrokerageRequest::ClosePositions).await;
    }

    pub async fn send_order(&self, order: Order) {
        self.send_request(BrokerageRequest::SendOrder(order)).await;
    }

    pub async fn subscribe(&self) -> UnboundedReceiver<Event> {
        let response = self.send_request(BrokerageRequest::Subscribe).await;
        if let BrokerageResponse::EventListener(receiver) = response {
            receiver
        } else {
            unreachable!()
        }
    }

    pub(crate) async fn reconcile_active_orders(&self) {
        self.send_request(BrokerageRequest::ReconcileOrders).await;
    }

    pub(crate) async fn expire_orders(&self) {
        self.send_request(BrokerageRequest::ExpireOrders).await;
    }

    // pub fn get_equity(&self) -> Decimal {
    //     let tickers = self.account.positions.keys();
    //     let positions_value: Decimal = tickers
    //         .map(|ticker| (ticker, self.market.get_current_price(ticker)))
    //         .map(|(ticker, price)| {
    //             self.account
    //                 .market_value(ticker, price.unwrap_or(Decimal::ZERO))
    //         })
    //         .sum();
    //     positions_value + self.account.cash
    // }
}
