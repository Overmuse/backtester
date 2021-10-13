use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderType {
    Market,
    Limit(Decimal),
    Stop(Decimal),
    StopLimit(Decimal, Decimal),
}

#[derive(Debug, Clone)]
pub struct Order {
    pub ticker: String,
    pub shares: Decimal,
    pub order_type: OrderType,
}

impl Order {
    pub fn new(ticker: String, shares: Decimal) -> Self {
        Self {
            ticker,
            shares,
            order_type: OrderType::Market,
        }
    }

    pub fn limit_price(mut self, limit_price: Decimal) -> Self {
        let order_type = match self.order_type {
            OrderType::Market | OrderType::Limit(_) => OrderType::Limit(limit_price),
            OrderType::Stop(stop_price) | OrderType::StopLimit(stop_price, _) => {
                OrderType::StopLimit(stop_price, limit_price)
            }
        };
        self.order_type = order_type;
        self
    }

    pub fn stop_price(mut self, stop_price: Decimal) -> Self {
        let order_type = match self.order_type {
            OrderType::Market | OrderType::Stop(_) => OrderType::Stop(stop_price),
            OrderType::Limit(limit_price) | OrderType::StopLimit(_, limit_price) => {
                OrderType::StopLimit(stop_price, limit_price)
            }
        };
        self.order_type = order_type;
        self
    }

    pub fn is_marketable(&self, price: Decimal) -> bool {
        match self.order_type {
            OrderType::Market => true,
            OrderType::Limit(limit_price) => {
                if self.shares.is_sign_positive() {
                    limit_price >= price
                } else {
                    limit_price <= price
                }
            }
            OrderType::Stop(stop_price) => {
                if self.shares.is_sign_positive() {
                    stop_price <= price
                } else {
                    stop_price >= price
                }
            }
            OrderType::StopLimit(stop_price, limit_price) => {
                if self.shares.is_sign_positive() {
                    stop_price <= price || limit_price >= price
                } else {
                    stop_price >= price || limit_price <= price
                }
            }
        }
    }
}
