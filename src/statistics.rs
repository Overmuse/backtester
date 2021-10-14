use rust_decimal::Decimal;

#[derive(Debug)]
pub struct Statistics {
    order_fills: usize,
    equity: Vec<Decimal>,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            order_fills: 0,
            equity: Vec::new(),
        }
    }

    pub fn increment_order_fills(&mut self) {
        self.order_fills += 1;
    }

    pub fn record_equity(&mut self, equity: Decimal) {
        self.equity.push(equity)
    }
}
