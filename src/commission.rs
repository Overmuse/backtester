use crate::order::Order;

pub trait Commission {
    fn calculate(&self, order: &Order) -> f64;
}

pub struct NoCommission;
impl Commission for NoCommission {
    fn calculate(&self, order: &Order) -> f64 {
        0.0
    }
}

pub struct PerShareCommission {
    amount: f64,
    min_order_cost: Option<f64>,
}
impl PerShareCommission {
    pub fn new(amount: f64) -> Self {
        Self {
            amount,
            min_order_cost: None,
        }
    }

    pub fn min_order_cost(mut self, min_order_cost: f64) -> Self {
        self.min_order_cost = Some(min_order_cost);
        self
    }
}
impl Commission for PerShareCommission {
    fn calculate(&self, order: &Order) -> f64 {
        f64::max(
            self.amount * order.shares,
            self.min_order_cost.unwrap_or(0.0),
        )
    }
}

pub struct PerOrderCommission {
    amount: f64,
}
impl PerOrderCommission {
    pub fn new(amount: f64) -> Self {
        Self { amount }
    }
}
impl Commission for PerOrderCommission {
    fn calculate(&self, order: &Order) -> f64 {
        self.amount
    }
}

pub struct PerDollarCommission {
    amount: f64,
    min_order_cost: Option<f64>,
}
impl PerDollarCommission {
    pub fn new(amount: f64) -> Self {
        Self {
            amount,
            min_order_cost: None,
        }
    }

    pub fn min_order_cost(mut self, min_order_cost: f64) -> Self {
        self.min_order_cost = Some(min_order_cost);
        self
    }
}
impl Commission for PerDollarCommission {
    fn calculate(&self, order: &Order) -> f64 {
        f64::max(
            self.amount * order.shares * order.price,
            self.min_order_cost.unwrap_or(0.0),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_calculates_the_correct_commission_amount() {
        let no_commission = NoCommission;
        let per_share_commission = PerShareCommission::new(1.0);
        let per_order_commission = PerOrderCommission::new(2.0);
        let per_dollar_commission = PerDollarCommission::new(3.0);

        let order = Order {
            ticker: "AAPL".to_string(),
            shares: 4.0,
            price: 5.0,
        };

        assert_eq!(no_commission.calculate(&order), 0.0);
        assert_eq!(per_share_commission.calculate(&order), 4.0);
        assert_eq!(per_order_commission.calculate(&order), 2.0);
        assert_eq!(per_dollar_commission.calculate(&order), 60.0);
    }

    #[test]
    fn it_respects_commission_minimums() {
        let per_share_commission = PerShareCommission::new(1.0).min_order_cost(5.0);
        let per_dollar_commission = PerDollarCommission::new(3.0).min_order_cost(100.0);

        let order = Order {
            ticker: "AAPL".to_string(),
            shares: 4.0,
            price: 5.0,
        };

        assert_eq!(per_share_commission.calculate(&order), 5.0);
        assert_eq!(per_dollar_commission.calculate(&order), 100.0);
    }
}
