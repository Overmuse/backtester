use crate::account::Account;
use crate::commission::{Commission, NoCommission};
//use crate::market::Market;
use crate::slippage::{NoSlippage, Slippage};

pub struct Brokerage<C: Commission, S: Slippage> {
    account: Account,
    //market: Market,
    commission: C,
    slippage: S,
}

impl Brokerage<NoCommission, NoSlippage> {
    pub fn new(cash: f64) -> Self {
        let account = Account::new(cash);
        Self {
            account,
            commission: NoCommission,
            slippage: NoSlippage,
        }
    }
}
