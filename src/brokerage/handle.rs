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
    CancelActiveOrders,
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

    #[tracing::instrument(skip(self, request))]
    async fn send_request(&self, request: BrokerageRequest) -> BrokerageResponse {
        let (tx, rx) = oneshot::channel();
        self.sender.send((tx, request)).unwrap();
        rx.await.unwrap()
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_positions(&self) -> Vec<Position> {
        let response = self.send_request(BrokerageRequest::GetPositions).await;
        if let BrokerageResponse::Positions(positions) = response {
            positions
        } else {
            unreachable!()
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_equity(&self) -> Decimal {
        let response = self.send_request(BrokerageRequest::GetEquity).await;
        if let BrokerageResponse::Decimal(equity) = response {
            equity
        } else {
            unreachable!()
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn close_positions(&self) {
        self.send_request(BrokerageRequest::ClosePositions).await;
    }

    #[tracing::instrument(skip(self))]
    pub async fn send_order(&self, order: Order) {
        self.send_request(BrokerageRequest::SendOrder(order)).await;
    }

    #[tracing::instrument(skip(self))]
    pub async fn subscribe(&self) -> UnboundedReceiver<Event> {
        let response = self.send_request(BrokerageRequest::Subscribe).await;
        if let BrokerageResponse::EventListener(receiver) = response {
            receiver
        } else {
            unreachable!()
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn cancel_active_orders(&self) {
        self.send_request(BrokerageRequest::CancelActiveOrders)
            .await;
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn reconcile_active_orders(&self) {
        self.send_request(BrokerageRequest::ReconcileOrders).await;
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn expire_orders(&self) {
        self.send_request(BrokerageRequest::ExpireOrders).await;
    }
}
