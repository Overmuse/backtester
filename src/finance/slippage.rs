pub trait Slippage: Send + Sync {
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
        if let Some(_max_volume) = self.max_volume {
            todo!("Implement max volume for VolumeShareSlippage")
        } else {
            self.price_impact * volume_share.powi(2)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_calculates_the_correct_slippage_amount() {
        let no_slippage = NoSlippage;
        let fixed_slippage = FixedSlippage::new(1.0);
        let volume_share_slippage = VolumeShareSlippage::new(2.0, None);

        assert_eq!(no_slippage.slippage(0.5), 0.0);
        assert_eq!(fixed_slippage.slippage(0.5), 1.0);
        assert_eq!(volume_share_slippage.slippage(0.5), 0.5);
    }
}
