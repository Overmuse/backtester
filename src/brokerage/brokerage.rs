use super::account::Account;
use super::order::Order;
use super::position::{Lot, Position};
use crate::finance::{
    commission::{Commission, NoCommission},
    slippage::{NoSlippage, Slippage},
};
use crate::markets::market::Market;
//use crate::market::Market;
use rust_decimal::Decimal;

pub struct Brokerage {
    account: Account,
    market: Market,
    commission: Box<dyn Commission>,
    slippage: Box<dyn Slippage>,
}

#[derive(Debug)]
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

    pub fn send_order(&mut self, order: Order) -> BrokerageOrder {
        let current_price = self.market.get_last_price(&order.ticker);
        if let Some(price) = current_price {
            if order.is_marketable(price) {
                let lot = Lot {
                    price,
                    quantity: order.shares,
                };
                self.account.add_lot(order.ticker.clone(), lot);
                return BrokerageOrder {
                    status: OrderStatus::Filled,
                    order,
                };
            }
        }
        BrokerageOrder {
            status: OrderStatus::Submitted,
            order,
        }
    }
}
