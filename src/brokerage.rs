use crate::account::Account;
use crate::commission::{Commission, NoCommission};
//use crate::market::Market;
use crate::slippage::{NoSlippage, Slippage};
use rust_decimal::Decimal;

pub struct Brokerage {
    account: Account,
    //market: Market,
    commission: Box<dyn Commission>,
    slippage: Box<dyn Slippage>,
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
}
