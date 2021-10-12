pub trait Slippage {
    fn slippage(&self, volume_share: f64) -> f64;
}

pub struct NoSlippage;

impl Slippage for NoSlippage {
    fn slippage(&self, _: f64) -> f64 {
        0.0
    }
}

pub struct FixedSlippage {
    amount: f64,
}

impl FixedSlippage {
    pub fn new(amount: f64) -> Self {
        Self { amount }
    }
}
impl Slippage for FixedSlippage {
    fn slippage(&self, _: f64) -> f64 {
        self.amount
    }
}

pub struct VolumeShareSlippage {
    price_impact: f64,
    max_volume: Option<f64>,
}
impl VolumeShareSlippage {
    pub fn new(price_impact: f64, max_volume: Option<f64>) -> Self {
        Self {
            price_impact,
            max_volume,
        }
    }
}
impl Slippage for VolumeShareSlippage {
    fn slippage(&self, volume_share: f64) -> f64 {
        if let Some(max_volume) = self.max_volume {
            todo!()
        } else {
            self.price_impact * volume_share.powi(2)
        }
    }
}
