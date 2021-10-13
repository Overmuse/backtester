use crate::account::Account;
use crate::finance::{
    commission::{Commission, NoCommission},
    slippage::{NoSlippage, Slippage},
};
use crate::order::Order;
//use crate::market::Market;
use rust_decimal::Decimal;

pub struct Brokerage {
    account: Account,
    //market: Market,
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
    pub fn new(cash: Decimal) -> Self {
        let account = Account::new(cash);
        Self {
            account,
            commission: Box::new(NoCommission),
            slippage: Box::new(NoSlippage),
        }
    }

    pub fn commission<C: 'static + Commission>(mut self, commission: C) -> Self {
        self.commission = Box::new(commission);
        self
    }

    pub fn send_order(&mut self, _: Order) {
        todo!()
    }
}
