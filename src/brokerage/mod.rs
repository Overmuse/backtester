use crate::finance::{
    commission::{Commission, NoCommission},
    slippage::{NoSlippage, Slippage},
};
use crate::markets::market::Market;
use account::Account;
use chrono::{DateTime, Utc};
use order::Order;
use position::{Lot, Position};
use rust_decimal::Decimal;
use std::sync::mpsc::{channel, Receiver, Sender};

pub mod account;
pub mod order;
pub mod position;

#[derive(Clone, Debug)]
pub enum Event {
    Commission {
        amount: Decimal,
    },
    OrderUpdate {
        status: OrderStatus,
        time: DateTime<Utc>,
        order: Order,
    },
}

pub struct Brokerage {
    account: Account,
    market: Market,
    commission: Box<dyn Commission>,
    slippage: Box<dyn Slippage>,
    listeners: Vec<Sender<Event>>,
}

#[derive(Clone, Debug)]
pub enum OrderStatus {
    Submitted,
    Cancelled,
    Filled {
        fill_time: DateTime<Utc>,
        average_fill_price: Decimal,
    },
    PartiallyFilled,
    Rejected,
}

#[derive(Debug)]
pub struct BrokerageOrder {
    status: OrderStatus,
    order: Order,
}

impl Brokerage {
    pub fn new(cash: Decimal, market: Market) -> Self {
        let account = Account::new(cash);
        Self {
            account,
            market,
            commission: Box::new(NoCommission),
            slippage: Box::new(NoSlippage),
            listeners: Vec::new(),
        }
    }

    pub fn get_account(&self) -> &Account {
        &self.account
    }

    pub fn get_positions(&self) -> Vec<&Position> {
        self.account.positions.values().collect()
    }

    pub fn commission<C: 'static + Commission>(mut self, commission: C) -> Self {
        self.commission = Box::new(commission);
        self
    }

    pub fn slippage<S: 'static + Slippage>(mut self, slippage: S) -> Self {
        self.slippage = Box::new(slippage);
        self
    }

    fn fill_order(&mut self, order: Order, price: Decimal) {
        let fill_time = self.market.datetime();
        let lot = Lot {
            fill_time,
            price,
            quantity: order.shares,
        };
        let commission = self.commission.calculate(&lot);
        self.account.add_lot(order.ticker.clone(), lot);
        self.account.cash -= commission;
        self.account.inactive_orders.push(order.clone());
        let time = self.market.datetime();
        let event = Event::OrderUpdate {
            status: OrderStatus::Filled {
                fill_time: time,
                average_fill_price: price,
            },
            time,
            order,
        };
        self.report_event(&event);
        let event = Event::Commission { amount: commission };
        self.report_event(&event);
    }

    fn save_order(&mut self, order: Order) {
        self.account.active_orders.push(order.clone());
        let event = Event::OrderUpdate {
            status: OrderStatus::Submitted,
            time: self.market.datetime(),
            order,
        };
        self.report_event(&event)
    }

    fn reject_order(&mut self, order: Order) {
        self.account.inactive_orders.push(order.clone());
        let event = Event::OrderUpdate {
            status: OrderStatus::Rejected,
            time: self.market.datetime(),
            order,
        };
        self.report_event(&event)
    }

    pub fn send_order(&mut self, order: Order) {
        if self.market.is_open() {
            let current_price = self.market.get_current_price(&order.ticker);
            if let Some(price) = current_price {
                if order.is_marketable(price) {
                    return self.fill_order(order, price);
                }
            }
            self.save_order(order)
        } else {
            self.reject_order(order);
        }
    }

    fn report_event(&self, event: &Event) {
        for listener in self.listeners.iter() {
            listener
                .send(event.clone())
                .expect("Failed to report event");
        }
    }

    pub(crate) fn reconcile_active_orders(&mut self) {
        // TODO: This whole function is very inefficient
        // Can clone cheaply here due to RC
        let market = self.market.clone();
        let orders_to_send: Vec<Order> = self
            .account
            .active_orders
            .drain_filter(|order| {
                let price = market.get_current_price(&order.ticker);
                match price {
                    Some(price) => order.is_marketable(price),
                    None => false,
                }
            })
            .collect();
        for order in orders_to_send {
            let price = market
                .get_current_price(&order.ticker)
                .expect("Guaranteed to exist");
            self.fill_order(order, price)
        }
    }

    pub fn subscribe(&mut self) -> Receiver<Event> {
        let (tx, rx) = channel();
        self.listeners.push(tx);
        rx
    }

    pub(crate) fn get_market(&self) -> Market {
        self.market.clone()
    }
}
