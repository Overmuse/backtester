use crate::finance::{
    commission::{Commission, NoCommission},
    slippage::{NoSlippage, Slippage},
};
use crate::markets::market::Market;
use account::Account;
use order::Order;
use position::{Lot, Position};
use rust_decimal::Decimal;
use std::sync::mpsc::{channel, Receiver, Sender};

pub mod account;
pub mod order;
pub mod position;

#[derive(Clone, Debug)]
pub enum Event {
    OrderUpdate { status: OrderStatus, order: Order },
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
    Filled,
    PartiallyFilled,
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

    pub fn send_order(&mut self, order: Order) {
        if self.market.is_open() {
            let current_price = self.market.get_last_price(&order.ticker);
            if let Some(price) = current_price {
                if order.is_marketable(price) {
                    let lot = Lot {
                        price,
                        quantity: order.shares,
                    };
                    self.account.add_lot(order.ticker.clone(), lot);
                    let event = Event::OrderUpdate {
                        status: OrderStatus::Filled,
                        order,
                    };
                    self.report_event(&event);
                    return;
                }
            }
        }
        let event = Event::OrderUpdate {
            status: OrderStatus::Submitted,
            order,
        };
        self.report_event(&event);
    }

    fn report_event(&self, event: &Event) {
        for listener in self.listeners.iter() {
            listener
                .send(event.clone())
                .expect("Failed to report event");
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
