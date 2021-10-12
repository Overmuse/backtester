use crate::order::Order;
use crate::position::Position;
use std::collections::HashMap;

pub struct Account {
    pub active_orders: Vec<Order>,
    pub inactive_orders: Vec<Order>,
    pub positions: HashMap<String, Position>,
    pub cash: f64,
    pub equity: f64,
    starting_cash: f64,
}

impl Account {
    pub fn new(cash: f64) -> Self {
        Self {
            active_orders: Vec::new(),
            inactive_orders: Vec::new(),
            positions: HashMap::new(),
            cash,
            equity: cash,
            starting_cash: cash,
        }
    }

    pub fn reset(&mut self) {
        self.cash = self.starting_cash;
        self.equity = self.starting_cash;
        self.active_orders.clear();
        self.inactive_orders.clear();
        self.positions.clear()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_can_be_initialized() {
        let account = Account::new(100.0);
        assert!(account.active_orders.is_empty());
        assert!(account.inactive_orders.is_empty());
        assert!(account.positions.is_empty());
        assert_eq!(account.cash, 100.0);
        assert_eq!(account.equity, 100.0);
        assert_eq!(account.starting_cash, 100.0);
    }

    #[test]
    fn it_can_be_reset() {
        let mut account = Account::new(100.0);
        account.cash = 200.0;
        account.reset();
        assert_eq!(account.cash, 100.0);
    }
}
