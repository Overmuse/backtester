use crate::brokerage::account::Account;
use crate::brokerage::handle::*;
use crate::brokerage::order::{Order, OrderStatus};
use crate::brokerage::position::Lot;
use crate::finance::{
    commission::{Commission, NoCommission},
    slippage::{NoSlippage, Slippage},
};
use crate::markets::handle::Market;
use chrono::DateTime;
use chrono_tz::Tz;
use futures::StreamExt;
use rust_decimal::Decimal;
use serde::Serialize;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::Sender as OneshotSender;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Commission {
        amount: Decimal,
    },
    OrderUpdate {
        status: OrderStatus,
        time: DateTime<Tz>,
        order: Order,
    },
}

pub struct BrokerageActor {
    requests: UnboundedReceiver<(OneshotSender<BrokerageResponse>, BrokerageRequest)>,
    account: Account,
    market: Market,
    commission: Box<dyn Commission>,
    _slippage: Box<dyn Slippage>,
    listeners: Vec<UnboundedSender<Event>>,
}

impl BrokerageActor {
    pub fn spawn(cash: Decimal, market: Market) -> Brokerage {
        let account = Account::new(cash);

        let (tx, rx) = unbounded_channel();
        let handle = Brokerage::new(tx);

        let actor = Self {
            requests: rx,
            account,
            market,
            commission: Box::new(NoCommission),
            _slippage: Box::new(NoSlippage),
            listeners: Vec::new(),
        };
        tokio::spawn(async move { actor.run_forever().await });
        handle
    }

    async fn run_forever(mut self) {
        while let Some((tx, request)) = self.requests.recv().await {
            let response = self.handle_message(request).await;
            tx.send(response).unwrap()
        }
    }

    async fn handle_message(&mut self, request: BrokerageRequest) -> BrokerageResponse {
        match request {
            BrokerageRequest::GetPositions => {
                BrokerageResponse::Positions(self.account.positions.values().cloned().collect())
            }
            BrokerageRequest::GetEquity => BrokerageResponse::Decimal(self.get_equity().await),
            BrokerageRequest::ClosePositions => {
                self.close_positions().await;
                BrokerageResponse::Success
            }
            BrokerageRequest::SendOrder(order) => {
                self.send_order(order).await;
                BrokerageResponse::Success
            }
            BrokerageRequest::ReconcileOrders => {
                self.reconcile_active_orders().await;
                BrokerageResponse::Success
            }
            BrokerageRequest::ExpireOrders => {
                self.expire_orders().await;
                BrokerageResponse::Success
            }
            BrokerageRequest::Subscribe => {
                let receiver = self.subscribe();
                BrokerageResponse::EventListener(receiver)
            }
        }
    }

    async fn get_equity(&self) -> Decimal {
        let tickers = self.account.positions.keys();
        let equity = futures::stream::iter(tickers)
            .fold(self.account.cash, |equity, ticker| async move {
                let price = self.market.get_current_price(ticker).await;
                let position_value = self
                    .account
                    .market_value(ticker, price.unwrap_or(Decimal::ZERO));
                equity + position_value
            })
            .await;
        equity
    }

    async fn close_positions(&mut self) {
        let orders: Vec<Order> = self
            .account
            .positions
            .values()
            .filter_map(|pos| {
                let qty = pos.quantity();
                if qty.is_zero() {
                    None
                } else {
                    Some(Order::new(pos.ticker.clone(), -qty))
                }
            })
            .collect();
        for order in orders {
            self.send_order(order).await
        }
        debug_assert_eq!(self.account.positions.values().count(), 0)
    }

    async fn send_order(&mut self, order: Order) {
        if self.market.is_open().await {
            let market = self.market.clone();
            let (_, current_price) = futures::join!(
                self.save_order(&order),
                market.get_current_price(&order.ticker)
            );
            if let Some(price) = current_price {
                if order.is_marketable(price) {
                    self.fill_order(order, price).await;
                }
            }
        } else {
            self.reject_order(order).await;
        }
    }

    async fn fill_order(&mut self, order: Order, price: Decimal) {
        let fill_time = self.market.datetime().await;
        let lot = Lot {
            fill_time,
            price,
            quantity: order.shares,
        };
        let commission = self.commission.calculate(&lot);
        self.account.add_lot(order.ticker.clone(), lot);
        self.account.cash -= commission;
        self.account.inactive_orders.push(order.clone());
        self.account.active_orders.retain(|o| o.id != order.id);
        let event = Event::OrderUpdate {
            status: OrderStatus::Filled {
                fill_time,
                average_fill_price: price,
            },
            time: fill_time,
            order,
        };
        self.report_event(&event);
        if !commission.is_zero() {
            let event = Event::Commission { amount: commission };
            self.report_event(&event);
        }
    }

    async fn save_order(&mut self, order: &Order) {
        self.account.active_orders.push(order.clone());
        let event = Event::OrderUpdate {
            status: OrderStatus::Submitted,
            time: self.market.datetime().await,
            order: order.clone(),
        };
        self.report_event(&event)
    }

    async fn reject_order(&mut self, order: Order) {
        self.account.inactive_orders.push(order.clone());
        let event = Event::OrderUpdate {
            status: OrderStatus::Rejected,
            time: self.market.datetime().await,
            order,
        };
        self.report_event(&event)
    }

    async fn expire_order(&mut self, order: Order) {
        self.account.inactive_orders.push(order.clone());
        let event = Event::OrderUpdate {
            status: OrderStatus::Expired,
            time: self.market.datetime().await,
            order,
        };
        self.report_event(&event)
    }

    fn report_event(&self, event: &Event) {
        for listener in self.listeners.iter() {
            listener
                .send(event.clone())
                .expect("Failed to report event");
        }
    }

    async fn reconcile_active_orders(&mut self) {
        // Manual version of drain_filter to be able to use the stable toolchain
        // TODO: Change to use drain_filter once https://github.com/rust-lang/rust/issues/43244 is
        // merged.
        let mut i = 0;
        let v = &mut self.account.active_orders;
        let mut orders_to_send: Vec<Order> = Vec::new();
        while i < v.len() {
            let order = &v[i];
            let price = self.market.get_current_price(&order.ticker).await;
            if let Some(price) = price {
                if order.is_marketable(price) {
                    let val = v.remove(i);
                    orders_to_send.push(val);
                } else {
                    i += 1
                }
            } else {
                i += 1
            }
        }
        for order in orders_to_send {
            let price = self
                .market
                .get_current_price(&order.ticker)
                .await
                .expect("Guaranteed to exist");
            self.fill_order(order, price).await
        }
    }

    async fn expire_orders(&mut self) {
        loop {
            let maybe_order = self.account.active_orders.pop();
            match maybe_order {
                Some(order) => self.expire_order(order).await,
                None => return,
            }
        }
    }

    fn subscribe(&mut self) -> UnboundedReceiver<Event> {
        let (tx, rx) = unbounded_channel();
        self.listeners.push(tx);
        rx
    }
}
