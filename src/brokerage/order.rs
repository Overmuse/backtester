use chrono::DateTime;
use chrono_tz::Tz;
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Submitted,
    Cancelled,
    Filled {
        fill_time: DateTime<Tz>,
        average_fill_price: Decimal,
    },
    PartiallyFilled,
    Rejected,
    Expired,
}

#[derive(Debug)]
pub struct BrokerageOrder {
    status: OrderStatus,
    order: Order,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Market,
    Limit(Decimal),
    Stop(Decimal),
    StopLimit(Decimal, Decimal),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Order {
    pub id: Uuid,
    pub ticker: String,
    pub shares: Decimal,
    pub order_type: OrderType,
}

impl Order {
    pub fn new<T: ToString>(ticker: T, shares: Decimal) -> Self {
        Self {
            id: Uuid::new_v4(),
            ticker: ticker.to_string(),
            shares: shares.round_dp(8),
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

    pub(crate) fn is_marketable(&self, price: Decimal) -> bool {
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
